use std::path::PathBuf;

use crate::models::track::Track;

#[derive(Clone, Debug)]
pub struct DownloadRequest {
    pub track: Track,
    pub output_dir: PathBuf,
    pub include_cover: bool,
    pub include_lyrics: bool,
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

impl DownloadRequest {
    pub fn single_track(track: Track, output_dir: PathBuf) -> Self {
        Self {
            track,
            output_dir,
            include_cover: true,
            include_lyrics: false,
        }
    }

    pub fn plan(self) -> DownloadPlan {
        let target_file = self.output_dir.join(format!(
            "{} - {}.m4a",
            sanitize_path_part(&self.track.artist),
            sanitize_path_part(&self.track.title)
        ));
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

fn sanitize_path_part(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
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

        assert_eq!(plan.target_file, PathBuf::from("/music/C_D - A_B.m4a"));
        assert_eq!(
            plan.sidecars,
            vec![DownloadSidecar::MetadataJson, DownloadSidecar::Cover]
        );
    }
}
