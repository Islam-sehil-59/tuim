use reqwest::header;

use crate::state::settings::AudioQuality;

use super::{
    client::ApiClient,
    playback::{PlaybackResolution, PlaybackSourceKind},
};

const MONOCHROME_REFERRER: &str = "https://monochrome.tf/";
const MONOCHROME_ORIGIN: &str = "https://monochrome.tf";

impl ApiClient {
    /// Full-length, non-Tidal stream lookup by ISRC, matching Monochrome's
    /// `LosslessAPI.getDeezerStreamUrl` (see `js/api.js`). Uses the configured
    /// Deezer fallback URL (configurable via settings) and selects formats
    /// based on the user's audio quality preference.
    pub async fn fetch_deezer_stream(
        &self,
        isrc: &str,
        quality: AudioQuality,
    ) -> Result<PlaybackResolution, String> {
        let base_url = self.deezer_fallback_base_url.trim_end_matches('/');
        let formats = quality.deezer_formats();
        let mut last_error = format!("{base_url}: no match for ISRC {isrc} in formats {formats:?}");

        for format in formats {
            let url = format!("{base_url}/stream/");

            let response = self
                .client
                .get(&url)
                .query(&[("isrc", isrc), ("format", format)])
                .header(header::REFERER, MONOCHROME_REFERRER)
                .header(header::ORIGIN, MONOCHROME_ORIGIN)
                .send()
                .await;

            match response {
                Ok(response) if response.status().is_success() => {
                    return Ok(PlaybackResolution {
                        instance: base_url.to_string(),
                        source_kind: PlaybackSourceKind::DeezerIsrcStream,
                        source: format!("{url}?isrc={isrc}&format={format}"),
                        audio_quality: Some(format.to_string()),
                        presentation: Some(String::from("FULL")),
                        preview_reason: None,
                        manifest_mime_type: None,
                        drm_protected: false,
                    });
                }
                Ok(response) => {
                    last_error = format!(
                        "{url} (isrc={isrc}, format={format}): {}",
                        response.status()
                    );
                }
                Err(error) => {
                    last_error = format!("{url} (isrc={isrc}, format={format}): {error}");
                }
            }
        }

        Err(last_error)
    }
}
