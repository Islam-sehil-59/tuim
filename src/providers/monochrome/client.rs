use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{Client, header};
use serde::de::DeserializeOwned;

use crate::models::track::Track;
use crate::state::settings::AudioQuality;

use super::{
    discovery,
    models::*,
    playback::{PlaybackResolution, PlaybackSourceKind},
    search::normalize_match_text,
};

const DEFAULT_DEEZER_FALLBACK_URL: &str = "https://dzr.tabs-vs-spaces.wtf";

#[derive(Clone)]
pub struct ApiClient {
    pub(super) client: Client,
    pub(super) deezer_fallback_base_url: String,
    last_working_instance: Arc<RwLock<Option<String>>>,
}

pub(super) struct RequestSpec {
    path: String,
    query: Vec<(String, String)>,
    accept_json_api: bool,
}

impl RequestSpec {
    pub(super) fn new(path: impl Into<String>, query: Vec<(String, String)>) -> Self {
        Self {
            path: path.into(),
            query,
            accept_json_api: false,
        }
    }

    pub(super) fn json_api(path: impl Into<String>, query: Vec<(String, String)>) -> Self {
        Self {
            path: path.into(),
            query,
            accept_json_api: true,
        }
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self::new_with_deezer_url(None)
    }

    pub fn new_with_deezer_url(deezer_fallback_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .expect("reqwest client should build");

        Self {
            client,
            deezer_fallback_base_url: deezer_fallback_url
                .unwrap_or_else(|| DEFAULT_DEEZER_FALLBACK_URL.to_string()),
            last_working_instance: Arc::new(RwLock::new(None)),
        }
    }

    pub fn deezer_fallback_base_url(&self) -> &str {
        &self.deezer_fallback_base_url
    }

    pub async fn fetch_stream_url(
        &self,
        track_id: u64,
        isrc: Option<&str>,
        quality: AudioQuality,
    ) -> Result<String, String> {
        Ok(self.resolve_playback(track_id, isrc, quality).await?.source)
    }

    pub async fn resolve_track_playback(
        &self,
        track: &Track,
        quality: AudioQuality,
    ) -> Result<PlaybackResolution, String> {
        let recovered_isrc;
        let isrc = if track.isrc.as_deref().is_some_and(|isrc| !isrc.is_empty()) {
            track.isrc.as_deref()
        } else {
            recovered_isrc = self.lookup_isrc_for_track(track).await.ok().flatten();
            recovered_isrc.as_deref()
        };

        self.resolve_playback(track.id, isrc, quality).await
    }

    /// Resolves a playable source for a track without returning a preview while
    /// a full-length source is still available. HiFi/TIDAL manifests stay first
    /// when their first media fragment is reachable; otherwise fall back to the
    /// non-TIDAL Deezer ISRC stream before returning a preview.
    pub async fn resolve_playback(
        &self,
        track_id: u64,
        isrc: Option<&str>,
        quality: AudioQuality,
    ) -> Result<PlaybackResolution, String> {
        let mut best_preview = None;
        let mut errors = Vec::new();

        match self.fetch_track_manifest_uri(track_id, quality).await {
            Ok(resolution) if resolution.is_full_playback() => {
                match self.probe_manifest_playable(&resolution).await {
                    Ok(()) => return Ok(resolution),
                    Err(error) => errors.push(error),
                }
            }
            Ok(resolution) => best_preview = Some(resolution),
            Err(error) => errors.push(error),
        }

        if let Some(isrc) = isrc
            && !isrc.is_empty()
        {
            match self.fetch_deezer_stream(isrc, quality).await {
                Ok(resolution) => return Ok(resolution),
                Err(error) => errors.push(error),
            }
        }

        match self.fetch_track_playback(track_id).await {
            Ok(resolution) if resolution.is_full_playback() => return Ok(resolution),
            Ok(resolution) if best_preview.is_none() => best_preview = Some(resolution),
            Ok(_) => {}
            Err(error) => errors.push(error),
        }

        best_preview.ok_or_else(|| errors.join("; "))
    }

    pub async fn probe_manifest_playable(
        &self,
        resolution: &PlaybackResolution,
    ) -> Result<(), String> {
        if !matches!(resolution.source_kind, PlaybackSourceKind::TrackManifestUri) {
            return Ok(());
        }

        let manifest = self
            .client
            .get(&resolution.source)
            .send()
            .await
            .map_err(|error| format!("HiFi manifest probe failed: {error}"))?;

        if !manifest.status().is_success() {
            return Err(format!(
                "HiFi manifest probe failed: {} returned {}",
                resolution.source,
                manifest.status()
            ));
        }

        let manifest_text = manifest
            .text()
            .await
            .map_err(|error| format!("HiFi manifest probe read failed: {error}"))?;
        let fragment_url = first_https_url(&manifest_text)
            .ok_or_else(|| String::from("HiFi manifest probe failed: no media URL found"))?;

        let fragment = self
            .client
            .get(&fragment_url)
            .header(header::RANGE, "bytes=0-0")
            .send()
            .await
            .map_err(|error| format!("HiFi fragment probe failed: {error}"))?;

        if fragment.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "HiFi fragment probe failed: {fragment_url} returned {}",
                fragment.status()
            ))
        }
    }

    async fn lookup_isrc_for_track(&self, track: &Track) -> Result<Option<String>, String> {
        let query = format!("{} {}", track.artist, track.title);
        let candidates = self.search_tracks(&query).await?;
        let target_title = normalize_match_text(&track.title);
        let target_artist = normalize_match_text(&track.artist);

        Ok(candidates
            .iter()
            .find(|candidate| candidate.id == track.id)
            .or_else(|| {
                candidates.iter().find(|candidate| {
                    normalize_match_text(&candidate.title) == target_title
                        && normalize_match_text(&candidate.artist) == target_artist
                })
            })
            .and_then(|candidate| candidate.isrc.clone()))
    }

    pub async fn fetch_track_manifest_uri(
        &self,
        track_id: u64,
        quality: AudioQuality,
    ) -> Result<PlaybackResolution, String> {
        let spec =
            RequestSpec::json_api("/trackManifests", tidal_manifest_query(track_id, quality));

        let mut best_preview = None;
        let mut last_error = String::from("all Monochrome instances failed");

        for instance in self.instance_candidates() {
            let payload: TrackManifestResponse =
                match self.request_json_on_instance(&instance, &spec).await {
                    Ok(payload) => payload,
                    Err(error) => {
                        last_error = error;
                        continue;
                    }
                };

            let attributes = payload.data.data.attributes;
            if attributes.drm_data.is_some() {
                last_error =
                    format!("{instance}: /trackManifests returned DRM-protected playback data");
                continue;
            }

            let resolution = PlaybackResolution {
                instance: instance.clone(),
                source_kind: PlaybackSourceKind::TrackManifestUri,
                source: attributes.uri,
                audio_quality: Some(tidal_quality_label(&attributes.formats).to_string()),
                presentation: Some(attributes.track_presentation.clone()),
                preview_reason: attributes.preview_reason,
                manifest_mime_type: Some(String::from("application/dash+xml")),
                drm_protected: false,
            };

            self.set_last_working_instance(&instance);
            if attributes.track_presentation == "FULL" {
                return Ok(resolution);
            }
            if best_preview.is_none() {
                best_preview = Some(resolution);
            }
        }

        best_preview.ok_or(last_error)
    }

    pub async fn fetch_track_playback(&self, track_id: u64) -> Result<PlaybackResolution, String> {
        let spec = RequestSpec::new("/track", vec![("id".to_string(), track_id.to_string())]);

        let mut best_preview = None;
        let mut last_error = String::from("all Monochrome instances failed");

        for instance in self.instance_candidates() {
            let payload: PlaybackResponse =
                match self.request_json_on_instance(&instance, &spec).await {
                    Ok(payload) => payload,
                    Err(error) => {
                        last_error = error;
                        continue;
                    }
                };

            let direct_url = payload.url().map(str::to_string);
            let Some(data) = payload.data else {
                last_error = format!("{instance}: track playback payload missing data");
                continue;
            };

            let resolution = if let Some(url) = direct_url {
                PlaybackResolution {
                    instance: instance.clone(),
                    source_kind: PlaybackSourceKind::DirectStreamUrl,
                    source: url,
                    audio_quality: Some(data.audio_quality.clone()),
                    presentation: Some(data.asset_presentation.clone()),
                    preview_reason: data.preview_reason.clone(),
                    manifest_mime_type: Some(data.manifest_mime_type.clone()),
                    drm_protected: false,
                }
            } else {
                let Some(manifest) = data.manifest else {
                    last_error =
                        format!("{instance}: track playback payload missing inline manifest");
                    continue;
                };
                let manifest_bytes = STANDARD
                    .decode(manifest)
                    .map_err(|error| error.to_string())?;
                let manifest_text =
                    String::from_utf8(manifest_bytes).map_err(|error| error.to_string())?;
                let path = Self::manifest_path(track_id);
                fs::write(&path, manifest_text).map_err(|error| error.to_string())?;

                PlaybackResolution {
                    instance: instance.clone(),
                    source_kind: PlaybackSourceKind::InlineManifestFile,
                    source: path.to_string_lossy().into_owned(),
                    audio_quality: Some(data.audio_quality.clone()),
                    presentation: Some(data.asset_presentation.clone()),
                    preview_reason: data.preview_reason.clone(),
                    manifest_mime_type: Some(data.manifest_mime_type.clone()),
                    drm_protected: false,
                }
            };

            self.set_last_working_instance(&instance);
            if data.asset_presentation == "FULL" {
                return Ok(resolution);
            }
            if best_preview.is_none() {
                best_preview = Some(resolution);
            }
        }

        best_preview.ok_or(last_error)
    }

    pub async fn fetch_direct_stream(&self, track_id: u64) -> Result<PlaybackResolution, String> {
        let (payload, instance): (PlaybackResponse, String) = self
            .request_json_with_instance(&[RequestSpec::new(
                "/stream",
                vec![
                    ("id".to_string(), track_id.to_string()),
                    ("quality".to_string(), "LOW".to_string()),
                ],
            )])
            .await?;

        let url = payload
            .url()
            .ok_or_else(|| String::from("/stream did not return a direct URL"))?;

        Ok(PlaybackResolution {
            instance,
            source_kind: PlaybackSourceKind::DirectStreamUrl,
            source: url.to_string(),
            audio_quality: None,
            presentation: None,
            preview_reason: None,
            manifest_mime_type: None,
            drm_protected: false,
        })
    }

    pub(super) async fn request_json<T>(&self, specs: &[RequestSpec]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        self.request_json_with_instance(specs)
            .await
            .map(|(payload, _)| payload)
    }

    pub(super) async fn request_json_with_instance<T>(
        &self,
        specs: &[RequestSpec],
    ) -> Result<(T, String), String>
    where
        T: DeserializeOwned,
    {
        let candidates = self.instance_candidates();
        let mut last_error = String::from("all Monochrome instances failed");

        for instance in candidates {
            for spec in specs {
                match self.request_json_on_instance::<T>(&instance, spec).await {
                    Ok(payload) => {
                        self.set_last_working_instance(&instance);
                        return Ok((payload, instance));
                    }
                    Err(error) => {
                        last_error = error;
                    }
                }
            }
        }

        Err(last_error)
    }

    pub(super) async fn request_json_on_instance<T>(
        &self,
        instance: &str,
        spec: &RequestSpec,
    ) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", instance.trim_end_matches('/'), spec.path);
        let mut request = self.client.get(&url).query(&spec.query);

        if spec.accept_json_api {
            request = request.header(
                header::ACCEPT,
                "application/vnd.api+json, application/json;q=0.9, */*;q=0.8",
            );
        }

        let response = request
            .send()
            .await
            .map_err(|error| format!("{url}: {error}"))?;

        if !response.status().is_success() {
            return Err(format!("{url}: {}", response.status()));
        }

        response
            .json::<T>()
            .await
            .map_err(|error| format!("{url}: {error}"))
    }

    fn instance_candidates(&self) -> Vec<String> {
        let mut candidates = Vec::new();

        if let Some(last) = self
            .last_working_instance
            .read()
            .ok()
            .and_then(|value| value.clone())
        {
            candidates.push(last);
        }

        for instance in discovery::default_instances() {
            if !candidates.iter().any(|candidate| candidate == instance) {
                candidates.push(instance.to_string());
            }
        }

        candidates
    }

    fn set_last_working_instance(&self, instance: &str) {
        if let Ok(mut last) = self.last_working_instance.write() {
            *last = Some(instance.to_string());
        }
    }

    fn manifest_path(track_id: u64) -> PathBuf {
        std::env::temp_dir().join(format!("tuim-{track_id}.mpd"))
    }
}

fn tidal_manifest_query(track_id: u64, quality: AudioQuality) -> Vec<(String, String)> {
    let formats = match quality {
        AudioQuality::HiResLossless => &["FLAC_HIRES"][..],
        AudioQuality::Lossless => &["FLAC"][..],
        AudioQuality::High => &["AACLC"][..],
        AudioQuality::Low => &["HEAACV1"][..],
        AudioQuality::Auto => &["FLAC_HIRES", "FLAC", "AACLC", "HEAACV1"][..],
    };
    let adaptive = matches!(quality, AudioQuality::Auto);

    let mut query = vec![("id".to_string(), track_id.to_string())];
    query.extend(
        formats
            .iter()
            .map(|format| ("formats".to_string(), (*format).to_string())),
    );
    query.extend([
        ("adaptive".to_string(), adaptive.to_string()),
        ("manifestType".to_string(), "MPEG_DASH".to_string()),
        ("uriScheme".to_string(), "HTTPS".to_string()),
        ("usage".to_string(), "PLAYBACK".to_string()),
    ]);
    query
}

fn tidal_quality_label(formats: &[String]) -> &'static str {
    if formats.iter().any(|format| format == "FLAC_HIRES") {
        "HI_RES_LOSSLESS"
    } else if formats.iter().any(|format| format == "FLAC") {
        "LOSSLESS"
    } else if formats.iter().any(|format| format == "AACLC") {
        "HIGH"
    } else if formats.iter().any(|format| format == "HEAACV1") {
        "LOW"
    } else {
        "UNKNOWN"
    }
}

fn first_https_url(value: &str) -> Option<String> {
    let start = value.find("https://")?;
    let rest = &value[start..];
    let end = rest
        .find(|character: char| {
            character == '"'
                || character == '\''
                || character == '<'
                || character == '>'
                || character.is_whitespace()
        })
        .unwrap_or(rest.len());
    Some(rest[..end].replace("&amp;", "&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_first_https_url_from_dash_manifest_text() {
        let manifest = r#"
            <MPD>
              <BaseURL>https://sp-ad-fa.audio.tidal.com/media/0.mp4?token=a&amp;info=b</BaseURL>
            </MPD>
        "#;

        assert_eq!(
            first_https_url(manifest).as_deref(),
            Some("https://sp-ad-fa.audio.tidal.com/media/0.mp4?token=a&info=b")
        );
    }

    #[test]
    fn tidal_manifest_query_uses_fixed_lossless_format() {
        let query = tidal_manifest_query(108316508, AudioQuality::Lossless);

        assert!(query.contains(&("formats".to_string(), "FLAC".to_string())));
        assert!(!query.contains(&("formats".to_string(), "AACLC".to_string())));
        assert!(query.contains(&("adaptive".to_string(), "false".to_string())));
    }

    #[test]
    fn tidal_manifest_query_keeps_auto_adaptive() {
        let query = tidal_manifest_query(108316508, AudioQuality::Auto);

        assert!(query.contains(&("formats".to_string(), "FLAC_HIRES".to_string())));
        assert!(query.contains(&("formats".to_string(), "FLAC".to_string())));
        assert!(query.contains(&("formats".to_string(), "AACLC".to_string())));
        assert!(query.contains(&("formats".to_string(), "HEAACV1".to_string())));
        assert!(query.contains(&("adaptive".to_string(), "true".to_string())));
    }

    #[test]
    fn tidal_quality_label_prefers_lossless_formats() {
        assert_eq!(
            tidal_quality_label(&[
                "HEAACV1".to_string(),
                "AACLC".to_string(),
                "FLAC".to_string()
            ]),
            "LOSSLESS"
        );
        assert_eq!(
            tidal_quality_label(&["FLAC_HIRES".to_string(), "FLAC".to_string()]),
            "HI_RES_LOSSLESS"
        );
    }
}
