use std::{fs, path::PathBuf};

use reqwest::Client;
use tokio::{fs as tokio_fs, task};

use crate::cache::paths;
use crate::models::{album::Album, artist::Artist, track::Track};

#[derive(Clone)]
pub struct ImageService {
    client: Client,
    cache_dir: PathBuf,
}

pub struct CoverArt {
    pub request_key: String,
    pub path: Option<String>,
}

impl ImageService {
    pub fn new() -> Self {
        let cache_dir = Self::cache_dir();
        let _ = fs::create_dir_all(&cache_dir);

        Self {
            client: Client::new(),
            cache_dir,
        }
    }

    pub async fn fetch_cover_for_track(&self, track: &Track) -> Result<CoverArt, String> {
        self.fetch_cover(
            format!("track:{}", track.id),
            track.cover_id.as_deref(),
            &[640, 750, 320],
        )
        .await
    }

    pub async fn fetch_cover_for_album(&self, album: &Album) -> Result<CoverArt, String> {
        self.fetch_cover(
            format!("album:{}", album.id),
            album.cover_id.as_deref(),
            &[640, 750, 320],
        )
        .await
    }

    pub async fn fetch_cover_for_artist(&self, artist: &Artist) -> Result<CoverArt, String> {
        self.fetch_cover(
            format!("artist:{}", artist.id),
            artist.picture_id.as_deref(),
            &[750, 640, 320],
        )
        .await
    }

    async fn fetch_cover(
        &self,
        request_key: String,
        cover_id: Option<&str>,
        sizes: &[u16],
    ) -> Result<CoverArt, String> {
        let Some(cover_id) = cover_id else {
            return Ok(CoverArt {
                request_key,
                path: None,
            });
        };
        let png_path = self
            .cache_dir
            .join(format!("{}.png", cover_id.replace('-', "_")));

        if !png_path.exists() {
            let jpg_bytes = self.fetch_cover_bytes(cover_id, sizes).await?;

            let png_bytes = task::spawn_blocking(move || -> Result<Vec<u8>, String> {
                let image =
                    image::load_from_memory(&jpg_bytes).map_err(|error| error.to_string())?;
                let mut png = Vec::new();
                image
                    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
                    .map_err(|error| error.to_string())?;
                Ok(png)
            })
            .await
            .map_err(|error| error.to_string())??;

            tokio_fs::write(&png_path, png_bytes)
                .await
                .map_err(|error| error.to_string())?;
        }

        Ok(CoverArt {
            request_key,
            path: Some(png_path.to_string_lossy().into_owned()),
        })
    }

    async fn fetch_cover_bytes(&self, cover_id: &str, sizes: &[u16]) -> Result<Vec<u8>, String> {
        let mut last_error = String::from("no image sizes configured");
        for size in sizes {
            let url = Self::cover_url(cover_id, *size);
            match self.client.get(&url).send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => {
                        return response
                            .bytes()
                            .await
                            .map(|bytes| bytes.to_vec())
                            .map_err(|error| error.to_string());
                    }
                    Err(error) => {
                        last_error = format!("{url}: {error}");
                    }
                },
                Err(error) => {
                    last_error = format!("{url}: {error}");
                }
            }
        }

        Err(last_error)
    }

    fn cache_dir() -> PathBuf {
        paths::covers_dir()
    }

    fn cover_url(cover_id: &str, size: u16) -> String {
        let formatted = cover_id.replace('-', "/");
        format!("https://resources.tidal.com/images/{formatted}/{size}x{size}.jpg")
    }
}
