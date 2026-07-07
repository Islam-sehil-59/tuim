# TODO

## Responsive TUI Layout

- Keep album cover art readable on home and lyrics screens.
- Add responsive layout modes for wide, medium, and narrow terminals.
- Home screen priority:
  - Keep status, search, results, cover, transport, and footer.
  - Hide preview before shrinking the cover.
  - Remove the recent/downloaded window from the primary layout.
- Lyrics screen priority:
  - Keep lyrics and cover visible.
  - Hide CAVA first when space is tight.
  - Move playback controls out of the cover column on medium/narrow layouts.
- Preserve mouse hit-testing after layout changes.
- Add layout unit tests for responsive mode selection and minimum cover sizing.
