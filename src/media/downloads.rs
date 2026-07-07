use std::path::PathBuf;

use crate::models::track::Track;

#[derive(Clone, Debug)]
pub struct DownloadRequest {
    pub track: Track,
    pub output_dir: PathBuf,
    pub include_cover: bool,
    pub include_lyrics: bool,
    pub collection: Option<String>,
    pub index: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct DownloadPlan {
    pub request: DownloadRequest,
    pub target_file: PathBuf,
    pub sidecars: Vec<DownloadSidecar>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DownloadSidecar {
    Cover,
    Lyrics,
    MetadataJson,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Resolving,
    Downloading { downloaded: u64, total: Option<u64> },
    WritingMetadata,
    Complete { path: PathBuf },
    Failed { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownloadedFile {
    pub track: Track,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownloadSummary {
    pub label: String,
    pub files: Vec<DownloadedFile>,
    pub failed: Vec<(Track, String)>,
    pub fatal_error: Option<String>,
}

impl DownloadRequest {
    pub fn single_track(track: Track, output_dir: PathBuf) -> Self {
        Self {
            track,
            output_dir,
            include_cover: true,
            include_lyrics: true,
            collection: None,
            index: None,
        }
    }

    pub fn plan(self) -> DownloadPlan {
        let target_file = target_path(&self, "m4a");
        let mut sidecars = vec![DownloadSidecar::MetadataJson];
        if self.include_cover {
            sidecars.push(DownloadSidecar::Cover);
        }
        if self.include_lyrics {
            sidecars.push(DownloadSidecar::Lyrics);
        }

        DownloadPlan {
            request: self,
            target_file,
            sidecars,
        }
    }
}

pub fn target_path(request: &DownloadRequest, extension: &str) -> PathBuf {
    let collection = request
        .collection
        .as_deref()
        .unwrap_or(request.track.album.as_str());
    let dir = request
        .output_dir
        .join(sanitize_path_part(&request.track.artist))
        .join(sanitize_path_part(collection));
    let prefix = request
        .index
        .map(|index| format!("{index:02} - "))
        .unwrap_or_default();

    dir.join(format!(
        "{prefix}{}.{}",
        sanitize_path_part(&request.track.title),
        extension.trim_start_matches('.')
    ))
}

pub fn sidecar_path(audio_path: &std::path::Path, suffix: &str) -> PathBuf {
    let stem = audio_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("metadata");
    audio_path.with_file_name(format!("{stem}.{suffix}"))
}

pub fn sanitize_path_part(value: &str) -> String {
    let sanitized = value
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>();

    if sanitized.is_empty() {
        String::from("unknown")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_track_download_plan_uses_safe_file_name_and_sidecars() {
        let track = Track {
            id: 1,
            title: "A/B".to_string(),
            artist: "C:D".to_string(),
            artist_id: 2,
            album: "Album".to_string(),
            album_id: 3,
            cover_id: None,
            isrc: None,
        };

        let plan = DownloadRequest::single_track(track, PathBuf::from("/music")).plan();

        assert_eq!(plan.target_file, PathBuf::from("/music/C_D/Album/A_B.m4a"));
        assert_eq!(
            plan.sidecars,
            vec![
                DownloadSidecar::MetadataJson,
                DownloadSidecar::Cover,
                DownloadSidecar::Lyrics
            ]
        );
    }

    #[test]
    fn target_path_can_include_track_index_and_collection() {
        let mut request = DownloadRequest::single_track(
            Track {
                id: 1,
                title: "Song".to_string(),
                artist: "Artist".to_string(),
                artist_id: 2,
                album: "Album".to_string(),
                album_id: 3,
                cover_id: None,
                isrc: None,
            },
            PathBuf::from("/music"),
        );
        request.collection = Some("Discography".to_string());
        request.index = Some(7);

        assert_eq!(
            target_path(&request, "flac"),
            PathBuf::from("/music/Artist/Discography/07 - Song.flac")
        );
    }
}
