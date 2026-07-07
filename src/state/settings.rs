use serde::{Deserialize, Serialize};

use crate::config::settings;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioQuality {
    #[default]
    Auto,
    HiResLossless,
    Lossless,
    High,
    Low,
}

impl AudioQuality {
    pub fn label(self) -> &'static str {
        match self {
            AudioQuality::Auto => "Auto",
            AudioQuality::HiResLossless => "Hi-Res Lossless",
            AudioQuality::Lossless => "Lossless",
            AudioQuality::High => "High",
            AudioQuality::Low => "Low",
        }
    }

    /// Deezer format preference order matching Monochrome's `getDeezerStreamFormat`.
    pub fn deezer_formats(self) -> &'static [&'static str] {
        match self {
            AudioQuality::Auto | AudioQuality::HiResLossless | AudioQuality::Lossless => {
                &["FLAC", "MP3_320"]
            }
            AudioQuality::High => &["MP3_320"],
            AudioQuality::Low => &["MP3_128"],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverDisplayMode {
    #[default]
    Cover,
    CoverRounded,
    #[serde(alias = "vinyl_spin")]
    VinylStill,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybarStyle {
    Classic,
    #[default]
    Modern,
}

impl PlaybarStyle {
    pub fn transport_height(self) -> u16 {
        match self {
            PlaybarStyle::Classic => 5,
            PlaybarStyle::Modern => 2,
        }
    }
}

impl CoverDisplayMode {
    pub fn label(self) -> &'static str {
        match self {
            CoverDisplayMode::Cover => "plain cover",
            CoverDisplayMode::CoverRounded => "rounded cover",
            CoverDisplayMode::VinylStill => "static vinyl cover",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SettingsState {
    #[serde(default)]
    pub active_theme: Option<String>,
    #[serde(default)]
    pub cover_display_mode: CoverDisplayMode,
    #[serde(default = "default_mouse_enabled")]
    pub mouse_enabled: bool,
    #[serde(default)]
    pub playbar_style: PlaybarStyle,
    #[serde(default)]
    pub audio_quality: AudioQuality,
    #[serde(default)]
    pub deezer_fallback_api_url: Option<String>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Self {
        settings::load().unwrap_or_default()
    }

    pub fn cycle_audio_quality(&mut self) {
        self.audio_quality = match self.audio_quality {
            AudioQuality::Auto => AudioQuality::HiResLossless,
            AudioQuality::HiResLossless => AudioQuality::Lossless,
            AudioQuality::Lossless => AudioQuality::High,
            AudioQuality::High => AudioQuality::Low,
            AudioQuality::Low => AudioQuality::Auto,
        };
    }

    pub fn cycle_cover_display_mode(&mut self) {
        self.cover_display_mode = match self.cover_display_mode {
            CoverDisplayMode::Cover => CoverDisplayMode::CoverRounded,
            CoverDisplayMode::CoverRounded => CoverDisplayMode::VinylStill,
            CoverDisplayMode::VinylStill => CoverDisplayMode::Cover,
        };
    }

    pub fn save(&self) -> Result<(), String> {
        settings::save(self)
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_theme: None,
            cover_display_mode: CoverDisplayMode::Cover,
            mouse_enabled: default_mouse_enabled(),
            playbar_style: PlaybarStyle::default(),
            audio_quality: AudioQuality::default(),
            deezer_fallback_api_url: None,
        }
    }
}

fn default_mouse_enabled() -> bool {
    true
}
