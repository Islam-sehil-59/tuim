use tuim::{
    media::playback::{PlaybackSource, PlaybackSourceKind},
    player::{error::PlayerError, mpv::PlaybackProgress, traits::Player},
};

#[derive(Default)]
struct FakePlayer {
    played: Vec<String>,
}

impl Player for FakePlayer {
    fn play(&mut self, source: &PlaybackSource) -> Result<(), PlayerError> {
        self.played.push(source.url.clone());
        Ok(())
    }
}

#[test]
fn player_trait_allows_backend_substitution() {
    let mut player = FakePlayer::default();
    let source = PlaybackSource::new(
        "https://example.test/audio.flac",
        PlaybackSourceKind::DirectUrl,
    );

    player.play(&source).unwrap();

    assert_eq!(player.played, vec!["https://example.test/audio.flac"]);
}

#[test]
fn playback_progress_formats_label_and_clamps_ratio() {
    let progress = PlaybackProgress {
        position_secs: 83.2,
        duration_secs: Some(240.0),
        ..PlaybackProgress::default()
    };

    assert_eq!(progress.label(), "1:23 / 4:00");
    assert!((progress.ratio() - 0.346).abs() < 0.01);

    let overrun = PlaybackProgress {
        position_secs: 999.0,
        duration_secs: Some(240.0),
        ..PlaybackProgress::default()
    };

    assert_eq!(overrun.ratio(), 1.0);
}
