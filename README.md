# TUIM — Terminal User Interface Music

[![GitHub stars](https://img.shields.io/github/stars/Islam-sehil-59/tuim?style=social)](https://github.com/Islam-sehil-59/tuim/stargazers)

A keyboard-driven terminal music player for Linux. Browse, search, stream, and
download music from the command line — zero GUI, all TUI.

TUIM is built for people who want a fast terminal-first music workflow: search
tracks, albums, and artists; queue what you find; play streams through `mpv`;
read synced lyrics; view cover art directly in compatible terminals; download
tracks or larger collections for local playback; and tune the interface with
JSON settings, themes, and keybindings.

## Features

- **Search & browse** TIDAL's catalogue (tracks, albums, artists)
- **Stream** at your chosen quality (MP3 / FLAC / HiRes)
- **Download** tracks, albums, and artist collections with metadata, cover, and lyrics sidecars
- **Queue** management with playlist-style navigation
- **Synced lyrics** with word-by-word karaoke highlighting
- **Album covers** rendered directly in the terminal (Kitty / Ghostty)
- **Audio visualizer** powered by [CAVA](https://github.com/karlstav/cava)
- **Custom themes** — JSON-based, hot-loadable
- **Custom keybinds** — fully rebindable via JSON config
- **Mouse support** — click to seek, scroll lists, adjust volume
- **mpv backend** — attach to an existing mpv instance or spawn a new one

## Quick start

```bash
# Install
cargo install --git https://github.com/Islam-sehil-59/tuim

# Launch
tuim
```

Requires **mpv** and optionally **CAVA** for the visualizer.

## How to use

Start TUIM and type a search query. Results are grouped by tracks, albums, and
artists. Press `Left` / `Right` to move between result tabs, `Enter` to play a
track or open an album/artist, and `Tab` to move focus between the search box
and results.

Common defaults:

- `Enter` searches, plays the selected track, or opens selected albums/artists
- `Space` pauses or resumes playback
- `Up` / `Down` moves through results or queue items
- `Left` / `Right` switches search result tabs or seeks while in playback views
- `Shift+Up` / `Shift+Down` changes volume
- `a` queues the selected track
- `Shift+A` queues the selected track next
- `Shift+P` plays all tracks in an opened album or artist view
- `Shift+Q` queues all tracks in an opened album or artist view
- `Shift+D` downloads the selected track, album, or artist collection
- `F1`, `F2`, `l`, and `F3` switch between Search, Queue, Lyrics, and Help

Downloaded tracks can be opened from inside TUIM by pressing `Enter` with an
empty search query. When a downloaded track is selected, TUIM plays the local
file instead of resolving a new stream.

## Configuration

Settings, keybinds, and themes live under `~/.config/tuim/`.

TUIM creates sensible defaults when files are missing. You can edit these files
while the app is closed:

```text
~/.config/tuim/settings.json
~/.config/tuim/keybinds.json
~/.config/tuim/themes/default.json
```

Downloads are stored under your XDG music directory when available, otherwise:

```text
~/Music/tuim/
```

Cache, covers, lyrics, logs, and generated cover variants are stored under:

```text
~/.cache/tuim/
```

## Themes

Themes are JSON files in `~/.config/tuim/themes/`. Set the active theme in
`settings.json`:

```json
{
  "active_theme": "midnight",
  "cover_display_mode": "cover_rounded",
  "playbar_style": "modern",
  "audio_quality": "auto",
  "mouse_enabled": true
}
```

Then create `~/.config/tuim/themes/midnight.json`:

```json
{
  "name": "midnight",
  "colors": {
    "background": "#151515",
    "foreground": "#edecec",
    "border": "#6f737a",
    "focused_border": "#ffffff",
    "muted_text": "#8a8a8a",
    "selected_text": "#151515",
    "selected_background": "#d8d8d8",
    "accent": "#3a8cff",
    "accent_secondary": "#48b84a",
    "progress_empty": "#444444",
    "progress_fill": "#3a8cff",
    "visualizer": "#3a8cff",
    "warning": "#d9a441",
    "error": "#ff5f5f",
    "success": "#48b84a"
  }
}
```

Supported colors can be `#rrggbb` values or terminal color names such as
`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`,
`dark_gray`, and the `light_*` variants.

## Keybindings

Keybindings are configured in `~/.config/tuim/keybinds.json`. Values use simple
strings such as `ctrl+u`, `shift+d`, `enter`, `space`, `left`, `right`, `up`,
`down`, or `f3`.

Example:

```json
{
  "quit": "esc",
  "clear_search": "ctrl+u",
  "download_selected": "shift+d",
  "play_selected": "enter",
  "queue_next_track": "n",
  "queue_previous_track": "p"
}
```

## Playback and downloads

Playback is handled by `mpv`. TUIM can attach to an existing mpv IPC socket or
spawn a new background mpv process. Cover rendering works best in terminals that
support the Kitty graphics protocol, such as Kitty and Ghostty.

Downloads include the audio file plus sidecars where available:

```text
Track.flac
Track.metadata.json
Track.cover.png
Track.lyrics.lrc
```

Metadata sidecars let TUIM restore downloaded tracks on the next launch and
play them from the local library view.

## License

MIT
