use std::fs;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

use crate::config::paths;

#[derive(Clone, Debug)]
pub struct Keybinds {
    pub search_view: KeyBinding,
    pub queue_view: KeyBinding,
    pub lyrics_view: KeyBinding,
    pub help_view: KeyBinding,
    pub help: KeyBinding,
    pub help_alt: KeyBinding,
    pub quit: KeyBinding,
    pub pause: KeyBinding,
    pub cover_mode: KeyBinding,
    pub audio_quality: KeyBinding,
    pub focus_search: KeyBinding,
    pub clear_search: KeyBinding,
    pub toggle_focus: KeyBinding,
    pub back: KeyBinding,
    pub filter_all: KeyBinding,
    pub filter_tracks: KeyBinding,
    pub filter_albums: KeyBinding,
    pub filter_artists: KeyBinding,
    pub play_all: KeyBinding,
    pub queue_all: KeyBinding,
    pub add_queue: KeyBinding,
    pub add_next: KeyBinding,
    pub stop: KeyBinding,
    pub seek_backward: KeyBinding,
    pub seek_forward: KeyBinding,
    pub volume_up: KeyBinding,
    pub volume_down: KeyBinding,
    pub mute: KeyBinding,
    pub download_selected: KeyBinding,
    pub select_previous: KeyBinding,
    pub select_next: KeyBinding,
    pub play_selected: KeyBinding,
    pub queue_remove: KeyBinding,
    pub queue_next_track: KeyBinding,
    pub queue_previous_track: KeyBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Debug, Deserialize)]
struct KeybindsFile {
    search_view: Option<String>,
    queue_view: Option<String>,
    lyrics_view: Option<String>,
    help_view: Option<String>,
    help: Option<String>,
    help_alt: Option<String>,
    quit: Option<String>,
    pause: Option<String>,
    cover_mode: Option<String>,
    #[serde(default)]
    audio_quality: Option<String>,
    focus_search: Option<String>,
    clear_search: Option<String>,
    toggle_focus: Option<String>,
    back: Option<String>,
    filter_all: Option<String>,
    filter_tracks: Option<String>,
    filter_albums: Option<String>,
    filter_artists: Option<String>,
    play_all: Option<String>,
    queue_all: Option<String>,
    add_queue: Option<String>,
    add_next: Option<String>,
    stop: Option<String>,
    seek_backward: Option<String>,
    seek_forward: Option<String>,
    volume_up: Option<String>,
    volume_down: Option<String>,
    mute: Option<String>,
    download_selected: Option<String>,
    select_previous: Option<String>,
    select_next: Option<String>,
    play_selected: Option<String>,
    queue_remove: Option<String>,
    queue_next_track: Option<String>,
    queue_previous_track: Option<String>,
}

impl Keybinds {
    pub fn load() -> Result<Self, String> {
        let path = paths::keybinds_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let file: KeybindsFile = serde_json::from_str(&text).map_err(|error| error.to_string())?;
        Self::from_file(file)
    }

    fn from_file(file: KeybindsFile) -> Result<Self, String> {
        let defaults = Self::default();

        Ok(Self {
            search_view: parse_or_default(file.search_view, defaults.search_view)?,
            queue_view: parse_or_default(file.queue_view, defaults.queue_view)?,
            lyrics_view: parse_or_default(file.lyrics_view, defaults.lyrics_view)?,
            help_view: parse_or_default(file.help_view, defaults.help_view)?,
            help: parse_or_default(file.help, defaults.help)?,
            help_alt: parse_or_default(file.help_alt, defaults.help_alt)?,
            quit: parse_or_default(file.quit, defaults.quit)?,
            pause: parse_or_default(file.pause, defaults.pause)?,
            cover_mode: parse_or_default(file.cover_mode, defaults.cover_mode)?,
            audio_quality: parse_or_default(file.audio_quality, defaults.audio_quality)?,
            focus_search: parse_or_default(file.focus_search, defaults.focus_search)?,
            clear_search: parse_or_default(file.clear_search, defaults.clear_search)?,
            toggle_focus: parse_or_default(file.toggle_focus, defaults.toggle_focus)?,
            back: parse_or_default(file.back, defaults.back)?,
            filter_all: parse_or_default(file.filter_all, defaults.filter_all)?,
            filter_tracks: parse_or_default(file.filter_tracks, defaults.filter_tracks)?,
            filter_albums: parse_or_default(file.filter_albums, defaults.filter_albums)?,
            filter_artists: parse_or_default(file.filter_artists, defaults.filter_artists)?,
            play_all: parse_or_default(file.play_all, defaults.play_all)?,
            queue_all: parse_or_default(file.queue_all, defaults.queue_all)?,
            add_queue: parse_or_default(file.add_queue, defaults.add_queue)?,
            add_next: parse_or_default(file.add_next, defaults.add_next)?,
            stop: parse_or_default(file.stop, defaults.stop)?,
            seek_backward: parse_or_default(file.seek_backward, defaults.seek_backward)?,
            seek_forward: parse_or_default(file.seek_forward, defaults.seek_forward)?,
            volume_up: parse_or_default(file.volume_up, defaults.volume_up)?,
            volume_down: parse_or_default(file.volume_down, defaults.volume_down)?,
            mute: parse_or_default(file.mute, defaults.mute)?,
            download_selected: parse_or_default(
                file.download_selected,
                defaults.download_selected,
            )?,
            select_previous: parse_or_default(file.select_previous, defaults.select_previous)?,
            select_next: parse_or_default(file.select_next, defaults.select_next)?,
            play_selected: parse_or_default(file.play_selected, defaults.play_selected)?,
            queue_remove: parse_or_default(file.queue_remove, defaults.queue_remove)?,
            queue_next_track: parse_or_default(file.queue_next_track, defaults.queue_next_track)?,
            queue_previous_track: parse_or_default(
                file.queue_previous_track,
                defaults.queue_previous_track,
            )?,
        })
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            search_view: KeyBinding::new(KeyCode::F(1), KeyModifiers::NONE),
            queue_view: KeyBinding::new(KeyCode::F(2), KeyModifiers::NONE),
            lyrics_view: KeyBinding::char('l'),
            help_view: KeyBinding::new(KeyCode::F(3), KeyModifiers::NONE),
            help: KeyBinding::char('h'),
            help_alt: KeyBinding::new(KeyCode::Char('?'), KeyModifiers::SHIFT),
            quit: KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE),
            pause: KeyBinding::new(KeyCode::Char(' '), KeyModifiers::NONE),
            cover_mode: KeyBinding::char('v'),
            audio_quality: KeyBinding::char('q'),
            focus_search: KeyBinding::char('/'),
            clear_search: KeyBinding::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            toggle_focus: KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE),
            back: KeyBinding::char('b'),
            filter_all: KeyBinding::char('1'),
            filter_tracks: KeyBinding::char('2'),
            filter_albums: KeyBinding::char('3'),
            filter_artists: KeyBinding::char('4'),
            play_all: KeyBinding::new(KeyCode::Char('p'), KeyModifiers::SHIFT),
            queue_all: KeyBinding::new(KeyCode::Char('q'), KeyModifiers::SHIFT),
            add_queue: KeyBinding::char('a'),
            add_next: KeyBinding::new(KeyCode::Char('a'), KeyModifiers::SHIFT),
            stop: KeyBinding::char('x'),
            seek_backward: KeyBinding::new(KeyCode::Left, KeyModifiers::NONE),
            seek_forward: KeyBinding::new(KeyCode::Right, KeyModifiers::NONE),
            volume_up: KeyBinding::new(KeyCode::Up, KeyModifiers::SHIFT),
            volume_down: KeyBinding::new(KeyCode::Down, KeyModifiers::SHIFT),
            mute: KeyBinding::char('m'),
            download_selected: KeyBinding::new(KeyCode::Char('d'), KeyModifiers::SHIFT),
            select_previous: KeyBinding::new(KeyCode::Up, KeyModifiers::NONE),
            select_next: KeyBinding::new(KeyCode::Down, KeyModifiers::NONE),
            play_selected: KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE),
            queue_remove: KeyBinding::char('d'),
            queue_next_track: KeyBinding::char('n'),
            queue_previous_track: KeyBinding::char('p'),
        }
    }
}

impl KeyBinding {
    fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    fn char(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        let mut modifiers = KeyModifiers::NONE;
        let parts: Vec<String> = value
            .split('+')
            .map(|part| part.trim().to_ascii_lowercase())
            .filter(|part| !part.is_empty())
            .collect();

        let Some(key) = parts.last() else {
            return Err(String::from("empty keybinding"));
        };

        for part in &parts[..parts.len().saturating_sub(1)] {
            match part.as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "alt" => modifiers |= KeyModifiers::ALT,
                _ => return Err(format!("unknown key modifier: {part}")),
            }
        }

        let code = match key.as_str() {
            "esc" | "escape" => KeyCode::Esc,
            "enter" | "return" => KeyCode::Enter,
            "tab" => KeyCode::Tab,
            "space" => KeyCode::Char(' '),
            "backspace" => KeyCode::Backspace,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            key if key.starts_with('f') => parse_function_key(key)?,
            key if key.chars().count() == 1 => KeyCode::Char(key.chars().next().unwrap()),
            _ => return Err(format!("unknown key: {key}")),
        };

        Ok(Self { code, modifiers })
    }

    pub fn matches(&self, event: KeyEvent) -> bool {
        if self.code == event.code && self.modifiers == event.modifiers {
            return true;
        }

        match (self.code, event.code) {
            (KeyCode::Char(expected), KeyCode::Char(actual))
                if expected.is_ascii_alphabetic()
                    && actual.eq_ignore_ascii_case(&expected)
                    && self.modifiers == KeyModifiers::SHIFT =>
            {
                event.modifiers == KeyModifiers::SHIFT
                    || (event.modifiers == KeyModifiers::NONE && actual.is_ascii_uppercase())
            }
            _ => false,
        }
    }
}

fn parse_or_default(
    value: Option<String>,
    default_binding: KeyBinding,
) -> Result<KeyBinding, String> {
    match value {
        Some(value) => KeyBinding::parse(&value),
        None => Ok(default_binding),
    }
}

fn parse_function_key(key: &str) -> Result<KeyCode, String> {
    let number = key[1..]
        .parse::<u8>()
        .map_err(|_| format!("invalid function key: {key}"))?;
    if number == 0 {
        return Err(format!("invalid function key: {key}"));
    }

    Ok(KeyCode::F(number))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_modified_and_character_bindings() {
        assert_eq!(
            KeyBinding::parse("f3").unwrap(),
            Keybinds::default().help_view
        );
        assert_eq!(
            KeyBinding::parse("ctrl+u").unwrap(),
            Keybinds::default().clear_search
        );
        assert_eq!(
            KeyBinding::parse("/").unwrap(),
            Keybinds::default().focus_search
        );
    }

    #[test]
    fn shifted_letter_bindings_match_common_terminal_events() {
        let binding = KeyBinding::parse("shift+p").unwrap();

        assert!(binding.matches(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::SHIFT)));
        assert!(binding.matches(KeyEvent::new(KeyCode::Char('P'), KeyModifiers::SHIFT)));
        assert!(binding.matches(KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE)));
        assert!(!binding.matches(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)));
    }
}
