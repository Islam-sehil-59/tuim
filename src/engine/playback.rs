use tokio::sync::mpsc::UnboundedSender;

use crate::{
    engine::events::RuntimeEvent,
    media::playback::PlaybackSource,
    models::track::Track,
    player::mpv::PlaybackExit,
    providers::{monochrome::MonochromeProvider, provider::MusicProvider},
    state::AppState,
};

#[derive(Default)]
pub struct PlaybackEngine {
    next_request_id: u64,
    pending_request_id: Option<u64>,
}

impl PlaybackEngine {
    pub fn new() -> Self {
        Self {
            next_request_id: 1,
            pending_request_id: None,
        }
    }

    pub fn begin_request(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.pending_request_id = Some(request_id);
        request_id
    }

    pub fn pending_request_id(&self) -> Option<u64> {
        self.pending_request_id
    }

    pub fn take_if_current(&mut self, request_id: u64) -> bool {
        if self.pending_request_id != Some(request_id) {
            return false;
        }

        self.pending_request_id = None;
        true
    }
}

pub fn spawn_playback_resolution(
    provider: MonochromeProvider,
    request_id: u64,
    track: Track,
    tx: UnboundedSender<RuntimeEvent>,
) {
    tokio::spawn(async move {
        let result = provider.resolve_playback(&track).await;
        let _ = tx.send(RuntimeEvent::PlaybackResolved {
            request_id,
            track,
            result,
        });
    });
}

pub struct PlaybackRequestStarted {
    pub request_id: u64,
    pub status_message: String,
    pub log_message: String,
}

pub struct PlaybackStarted {
    pub status_message: String,
    pub preview_reason: Option<String>,
}

pub struct PlaybackStopped {
    pub status_message: String,
    pub preview_reason: Option<String>,
}

pub enum PlaybackExitApply {
    Ended {
        status_message: String,
        log_message: String,
    },
    PreviewEnded {
        status_message: String,
        log_message: String,
    },
    Failed {
        status_message: String,
        log_message: String,
    },
}

pub fn begin_playback_request(
    state: &mut AppState,
    engine: &mut PlaybackEngine,
    track: &Track,
) -> PlaybackRequestStarted {
    let request_id = engine.begin_request();
    let status_message = format!("Resolving stream for {} — {}...", track.artist, track.title);
    state.status.message = status_message.clone();

    PlaybackRequestStarted {
        request_id,
        status_message,
        log_message: format!(
            "playback requested request_id={request_id} track_id={} artist={} title={}",
            track.id, track.artist, track.title
        ),
    }
}

pub fn apply_playback_started(
    state: &mut AppState,
    track: Track,
    source: &PlaybackSource,
) -> PlaybackStarted {
    state.player.now_playing = Some(track.clone());
    state.player.progress = None;
    state.player.paused = false;
    state.player.attached_playback = true;
    state.player.source_quality = source.quality.clone();

    let status_message = if let Some(reason) = &source.preview_reason {
        format!(
            "Playing preview: {} — {} ({reason})",
            track.artist, track.title
        )
    } else {
        format!("Playing: {} — {}", track.artist, track.title)
    };
    state.status.message = status_message.clone();

    PlaybackStarted {
        status_message,
        preview_reason: source.preview_reason.clone(),
    }
}

pub fn apply_playback_start_failed(state: &mut AppState, error: impl Into<String>) -> String {
    let error = error.into();
    state.player.progress = None;
    state.player.source_quality = None;
    state.lyrics.clear();
    state.status.message = format!("mpv failed: {error}");
    state.status.message.clone()
}

pub fn apply_playback_resolution_failed(state: &mut AppState, error: impl Into<String>) -> String {
    let error = error.into();
    state.player.progress = None;
    state.player.source_quality = None;
    state.lyrics.clear();
    state.status.message = format!("Stream lookup failed: {error}");
    state.status.message.clone()
}

pub fn apply_pause_toggled(state: &mut AppState) -> String {
    state.player.paused = !state.player.paused;
    state.status.message = if state.player.paused {
        String::from("Playback paused.")
    } else {
        String::from("Playback resumed.")
    };
    state.status.message.clone()
}

pub fn apply_playback_stopped(state: &mut AppState) -> PlaybackStopped {
    state.player.now_playing = None;
    state.player.progress = None;
    state.player.paused = false;
    state.player.attached_playback = false;
    state.player.source_quality = None;
    state.lyrics.clear();
    state.status.message = String::from("Playback stopped.");

    PlaybackStopped {
        status_message: state.status.message.clone(),
        preview_reason: None,
    }
}

pub fn apply_player_exit(
    state: &mut AppState,
    exit: PlaybackExit,
    preview_reason: Option<String>,
) -> PlaybackExitApply {
    state.player.progress = None;
    state.player.paused = false;
    state.player.attached_playback = false;
    state.player.source_quality = None;

    if let Some(reason) = preview_reason {
        state.lyrics.clear();
        state.status.message =
            format!("Playback ended: preview-only stream stopped normally ({reason}).");
        return PlaybackExitApply::PreviewEnded {
            status_message: state.status.message.clone(),
            log_message: format!("mpv exit observed after preview-only playback reason={reason}"),
        };
    }

    state.lyrics.clear();
    state.status.message = exit.message.clone();
    if exit.success {
        PlaybackExitApply::Ended {
            status_message: exit.message.clone(),
            log_message: format!("mpv exit observed message={}", exit.message),
        }
    } else {
        PlaybackExitApply::Failed {
            status_message: exit.message.clone(),
            log_message: format!("mpv exit observed message={}", exit.message),
        }
    }
}
