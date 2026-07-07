# TUIM — Terminal User Interface Music

A keyboard-driven terminal music player for Linux. Browse, search, stream, and
visualize audio from the command line — zero GUI, all TUI.

## Features

- **Search & browse** TIDAL's catalogue (tracks, albums, artists)
- **Stream** at your chosen quality (MP3 / FLAC / HiRes)
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

## Configuration

Settings, keybinds, and themes live under `~/.config/tuim/`.

## License

MIT
