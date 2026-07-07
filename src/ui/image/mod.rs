use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{Rgba, RgbaImage, image_dimensions, imageops::FilterType};
use ratatui::layout::Rect;

use crate::cache::paths;
use crate::state::{
    AppState,
    settings::{CoverDisplayMode, SettingsState},
    view::View,
};

const COVER_IMAGE_ID: u32 = 1;
const VINYL_FRAME_SIZE: u32 = 512;
const VINYL_CACHE_VERSION: &str = "static-cover-circle-v1";
const RENDERED_FRAME_SIZE: u32 = 512;
const ROUNDED_CACHE_VERSION: &str = "rounded-corners-v1";
const CORNER_RADIUS_RATIO: f32 = 0.06;
const TERMINAL_CELL_ASPECT_RATIO: f32 = 0.5;

pub struct KittyImageRenderer {
    supported: bool,
    last_path: Option<String>,
    last_rect: Option<Rect>,
}

impl KittyImageRenderer {
    pub fn new() -> Self {
        Self {
            supported: Self::detect_support(),
            last_path: None,
            last_rect: None,
        }
    }

    pub fn is_supported(&self) -> bool {
        self.supported
    }

    fn detect_support() -> bool {
        if env::var("TERM")
            .map(|value| value.eq_ignore_ascii_case("dumb"))
            .unwrap_or(false)
        {
            return false;
        }

        if env::var("TERM_PROGRAM")
            .map(|value| {
                value.eq_ignore_ascii_case("ghostty") || value.eq_ignore_ascii_case("kitty")
            })
            .unwrap_or(false)
        {
            return true;
        }

        if env::var("TERM")
            .map(|value| value.contains("ghostty") || value.contains("kitty"))
            .unwrap_or(false)
        {
            return true;
        }

        true
    }

    pub fn sync_cover(&mut self, cover_rect: Rect, state: &AppState) -> io::Result<()> {
        if !self.supported {
            return Ok(());
        }

        let inner = Self::inner_rect(cover_rect);
        let cover = if state.current_view == View::Lyrics {
            &state.playback_cover
        } else {
            &state.cover
        };
        let target_path = if cover.loading {
            None
        } else {
            cover.path.as_deref()
        };

        match target_path {
            Some(path) if inner.width > 0 && inner.height > 0 => {
                let display_path = display_path_for_mode(path, &state.settings)
                    .unwrap_or_else(|| path.to_string());
                let display_rect = Self::aspect_fit_rect(inner, &display_path)
                    .unwrap_or_else(|_| Self::aspect_fit_rect_for_ratio(inner, 1.0));

                if self.last_path.as_deref() == Some(display_path.as_str())
                    && self.last_rect == Some(display_rect)
                {
                    return Ok(());
                }

                if self.last_path.is_some() || self.last_rect.is_some() {
                    self.hide()?;
                }
                self.display(&display_path, display_rect)?;
                self.last_path = Some(display_path);
                self.last_rect = Some(display_rect);
            }
            _ => {
                self.hide()?;
            }
        }

        Ok(())
    }

    fn display(&mut self, path: &str, rect: Rect) -> io::Result<()> {
        let row = rect.y + 1;
        let col = rect.x + 1;
        let payload = STANDARD.encode(path.as_bytes());
        let mut stdout = io::stdout();

        write!(
            stdout,
            "\x1b[s\x1b[{row};{col}H\x1b_Ga=T,i={id},f=100,t=f,c={cols},C=1,q=2;{payload}\x1b\\\x1b[u",
            id = COVER_IMAGE_ID,
            cols = rect.width,
        )?;
        stdout.flush()?;
        Ok(())
    }

    pub fn hide(&mut self) -> io::Result<()> {
        if self.last_path.is_none() && self.last_rect.is_none() {
            return Ok(());
        }

        let mut stdout = io::stdout();
        write!(
            stdout,
            "\x1b_Ga=d,d=I,i={id},q=2\x1b\\",
            id = COVER_IMAGE_ID
        )?;
        stdout.flush()?;
        self.last_path = None;
        self.last_rect = None;
        Ok(())
    }

    fn inner_rect(rect: Rect) -> Rect {
        Rect {
            x: rect.x.saturating_add(1),
            y: rect.y.saturating_add(1),
            width: rect.width.saturating_sub(2),
            height: rect.height.saturating_sub(2),
        }
    }

    fn aspect_fit_rect(rect: Rect, path: &str) -> io::Result<Rect> {
        let (width, height) = image_dimensions(path).map_err(to_io_error)?;
        if width == 0 || height == 0 {
            return Ok(rect);
        }

        Ok(Self::aspect_fit_rect_for_ratio(
            rect,
            width as f32 / height as f32,
        ))
    }

    fn aspect_fit_rect_for_ratio(rect: Rect, image_aspect_ratio: f32) -> Rect {
        if rect.width == 0 || rect.height == 0 {
            return rect;
        }

        let cell_ratio = (image_aspect_ratio / TERMINAL_CELL_ASPECT_RATIO).max(0.1);
        let max_width = f32::from(rect.width);
        let max_height = f32::from(rect.height);
        let mut target_width = max_width;
        let mut target_height = target_width / cell_ratio;

        if target_height > max_height {
            target_height = max_height;
            target_width = target_height * cell_ratio;
        }

        let target_width = (target_width.round() as u16).clamp(1, rect.width);
        let target_height = (target_height.round() as u16).clamp(1, rect.height);

        Rect {
            x: rect.x + rect.width.saturating_sub(target_width) / 2,
            y: rect.y + rect.height.saturating_sub(target_height) / 2,
            width: target_width,
            height: target_height,
        }
    }
}

impl Default for KittyImageRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn display_path_for_mode(source: &str, settings: &SettingsState) -> Option<String> {
    match settings.cover_display_mode {
        CoverDisplayMode::Cover => Some(source.to_string()),
        CoverDisplayMode::CoverRounded => {
            let path = rounded_path(source).ok()?;
            Some(path.to_string_lossy().into_owned())
        }
        CoverDisplayMode::VinylStill => {
            let path = vinyl_path(source).ok()?;
            Some(path.to_string_lossy().into_owned())
        }
    }
}

fn vinyl_path(source: &str) -> io::Result<PathBuf> {
    let cache_key = vinyl_cache_key(source);
    let vinyl_dir = paths::vinyl_dir();
    fs::create_dir_all(&vinyl_dir)?;
    let path = vinyl_dir.join(format!("{cache_key:016x}.png"));
    if !path.exists() {
        render_vinyl(source, &path)?;
    }

    Ok(path)
}

fn vinyl_cache_key(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    VINYL_CACHE_VERSION.hash(&mut hasher);
    source.hash(&mut hasher);
    if let Ok(metadata) = fs::metadata(source) {
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified()
            && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
        {
            duration.as_secs().hash(&mut hasher);
        }
    }

    hasher.finish()
}

fn render_vinyl(source: &str, output: &Path) -> io::Result<()> {
    let image = image::open(source).map_err(to_io_error)?;
    let square = image
        .resize_to_fill(VINYL_FRAME_SIZE, VINYL_FRAME_SIZE, FilterType::Lanczos3)
        .to_rgba8();
    let mut output_image = RgbaImage::new(VINYL_FRAME_SIZE, VINYL_FRAME_SIZE);
    let center = (VINYL_FRAME_SIZE as f32 - 1.0) / 2.0;
    let outer_radius = center;
    let label_radius = VINYL_FRAME_SIZE as f32 * 0.115;

    for y in 0..VINYL_FRAME_SIZE {
        for x in 0..VINYL_FRAME_SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let radius = (dx * dx + dy * dy).sqrt();

            if radius > outer_radius {
                output_image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            if radius <= label_radius {
                output_image.put_pixel(x, y, Rgba([245, 245, 245, 255]));
                continue;
            }

            let mut pixel = *square.get_pixel(x, y);
            pixel.0[3] = 255;

            output_image.put_pixel(x, y, pixel);
        }
    }

    output_image.save(output).map_err(to_io_error)
}

fn rounded_path(source: &str) -> io::Result<PathBuf> {
    let cache_key = rounded_cache_key(source);
    let vinyl_dir = paths::vinyl_dir();
    fs::create_dir_all(&vinyl_dir)?;
    let path = vinyl_dir.join(format!("rounded_{cache_key:016x}.png"));
    if !path.exists() {
        render_rounded(source, &path)?;
    }

    Ok(path)
}

fn rounded_cache_key(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    ROUNDED_CACHE_VERSION.hash(&mut hasher);
    source.hash(&mut hasher);
    if let Ok(metadata) = fs::metadata(source) {
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified()
            && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
        {
            duration.as_secs().hash(&mut hasher);
        }
    }

    hasher.finish()
}

fn render_rounded(source: &str, output: &Path) -> io::Result<()> {
    let image = image::open(source).map_err(to_io_error)?;
    let size = RENDERED_FRAME_SIZE;
    let square = image
        .resize_to_fill(size, size, FilterType::Lanczos3)
        .to_rgba8();
    let mut output_image = RgbaImage::new(size, size);
    let corner_radius = size as f32 * CORNER_RADIUS_RATIO;

    for y in 0..size {
        for x in 0..size {
            let dx = if x < corner_radius as u32 {
                (corner_radius - x as f32).max(0.0)
            } else if x > size - corner_radius as u32 - 1 {
                (x as f32 - (size as f32 - 1.0 - corner_radius)).max(0.0)
            } else {
                0.0
            };

            let dy = if y < corner_radius as u32 {
                (corner_radius - y as f32).max(0.0)
            } else if y > size - corner_radius as u32 - 1 {
                (y as f32 - (size as f32 - 1.0 - corner_radius)).max(0.0)
            } else {
                0.0
            };

            let distance = (dx * dx + dy * dy).sqrt();
            let mut pixel = *square.get_pixel(x, y);
            pixel.0[3] = if distance > corner_radius { 0 } else { 255 };

            output_image.put_pixel(x, y, pixel);
        }
    }

    output_image.save(output).map_err(to_io_error)
}

fn to_io_error(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::other(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspect_fit_keeps_square_images_square_in_terminal_cells() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };

        let fitted = KittyImageRenderer::aspect_fit_rect_for_ratio(rect, 1.0);

        assert_eq!(fitted.width, 40);
        assert_eq!(fitted.height, 20);
    }

    #[test]
    fn aspect_fit_centers_wide_images_without_stretching() {
        let rect = Rect {
            x: 2,
            y: 3,
            width: 40,
            height: 20,
        };

        let fitted = KittyImageRenderer::aspect_fit_rect_for_ratio(rect, 2.0);

        assert_eq!(fitted.width, 40);
        assert_eq!(fitted.height, 10);
        assert_eq!(fitted.x, 2);
        assert_eq!(fitted.y, 8);
    }
}
