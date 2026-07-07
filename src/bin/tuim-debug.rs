use std::{
    env,
    process::{Command, Stdio},
};

use tuim::{
    providers::monochrome::{client::ApiClient, playback::PlaybackSourceKind},
    services::image::ImageService,
    state::settings::AudioQuality,
};

#[tokio::main]
async fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("artist") {
        let Some(artist_id) = args.get(1).and_then(|value| value.parse::<u64>().ok()) else {
            eprintln!("usage: cargo run --bin tuim-debug -- artist <artist-id>");
            std::process::exit(2);
        };
        debug_artist(artist_id).await;
        return;
    }

    let Some(track_id) = args.first().and_then(|value| value.parse::<u64>().ok()) else {
        eprintln!(
            "usage: cargo run --bin tuim-debug -- <track-id> [--isrc <isrc>] [--play] [--probe]\n       cargo run --bin tuim-debug -- artist <artist-id>"
        );
        std::process::exit(2);
    };

    let mut should_play = false;
    let mut probe_only = false;
    let mut isrc: Option<String> = None;
    let mut args = args.into_iter().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--play" => should_play = true,
            "--probe" => probe_only = true,
            "--isrc" => {
                isrc = args.next();
                if isrc.is_none() {
                    eprintln!("--isrc requires a value");
                    std::process::exit(2);
                }
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    let api = ApiClient::new();

    println!(
        "== Deezer ISRC probe (non-Tidal, full song, same method as the Monochrome repo's fallback) =="
    );
    match &isrc {
        Some(isrc) => match api.fetch_deezer_stream(isrc, AudioQuality::Lossless).await {
            Ok(stream) => {
                println!("instance: {}", stream.instance);
                println!("kind: {:?}", stream.source_kind);
                println!("source: {}", stream.source);
                println!("quality: {:?}", stream.audio_quality);
                println!("presentation: {:?}", stream.presentation);
            }
            Err(error) => println!("error: {error}"),
        },
        None => println!("skipped: pass --isrc <isrc> to test this path directly"),
    }

    println!();
    println!("== direct /stream probe (low quality, debug only) ==");
    match api.fetch_direct_stream(track_id).await {
        Ok(stream) => {
            println!("instance: {}", stream.instance);
            println!("kind: {:?}", stream.source_kind);
            println!("source: {}", stream.source);
        }
        Err(error) => println!("error: {error}"),
    }

    println!();
    println!("== /trackManifests probe (Tidal, subscription-gated preview fallback) ==");
    match api
        .fetch_track_manifest_uri(track_id, AudioQuality::Lossless)
        .await
    {
        Ok(manifest) => {
            println!("instance: {}", manifest.instance);
            println!("kind: {:?}", manifest.source_kind);
            println!("source: {}", manifest.source);
            println!("quality: {:?}", manifest.audio_quality);
            println!("presentation: {:?}", manifest.presentation);
            println!("preview_reason: {:?}", manifest.preview_reason);
            println!("drm: {}", manifest.drm_protected);
            match api.probe_manifest_playable(&manifest).await {
                Ok(()) => println!("playability_probe: ok"),
                Err(error) => println!("playability_probe: {error}"),
            }
        }
        Err(error) => println!("error: {error}"),
    }

    println!();
    println!("== resolved playback used by TUI (HiFi when playable, Deezer fallback otherwise) ==");
    let resolution = match api
        .resolve_playback(track_id, isrc.as_deref(), AudioQuality::Lossless)
        .await
    {
        Ok(resolution) => {
            println!("instance: {}", resolution.instance);
            println!("kind: {:?}", resolution.source_kind);
            println!("source: {}", resolution.source);
            println!("quality: {:?}", resolution.audio_quality);
            println!("presentation: {:?}", resolution.presentation);
            println!("preview_reason: {:?}", resolution.preview_reason);
            println!("manifest_mime_type: {:?}", resolution.manifest_mime_type);
            resolution
        }
        Err(error) => {
            eprintln!("failed to resolve playback: {error}");
            std::process::exit(1);
        }
    };

    if !should_play && !probe_only {
        return;
    }

    println!();
    println!("== mpv test ==");
    let mut command = Command::new("mpv");
    command
        .arg("--no-video")
        .arg("--force-window=no")
        .arg("--msg-level=all=info")
        .arg("--demuxer-lavf-o=protocol_whitelist=[file,http,https,tcp,tls,crypto]");

    if matches!(
        resolution.source_kind,
        PlaybackSourceKind::DeezerIsrcStream | PlaybackSourceKind::DirectStreamUrl
    ) {
        command
            .arg("--http-header-fields=Referer: https://monochrome.tf/")
            .arg("--http-header-fields=Origin: https://monochrome.tf");
    }

    command.arg(&resolution.source);

    if probe_only {
        command
            .arg("--ao=null")
            .arg("--length=3")
            .arg("--really-quiet=no");
    }

    println!("spawning: {:?}", command);

    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => {
            println!("mpv exit status: {status}");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Err(error) => {
            eprintln!("failed to spawn mpv: {error}");
            std::process::exit(1);
        }
    }
}

async fn debug_artist(artist_id: u64) {
    let api = ApiClient::new();
    let image = ImageService::new();

    println!("== artist details ==");
    let details = match api.fetch_artist_details(artist_id).await {
        Ok(details) => details,
        Err(error) => {
            eprintln!("artist detail failed: {error}");
            std::process::exit(1);
        }
    };

    println!("id: {}", details.artist.id);
    println!("name: {}", details.artist.name);
    println!("picture_id: {:?}", details.artist.picture_id);
    println!("albums: {}", details.albums.len());
    println!("tracks: {}", details.tracks.len());
    for album in details.albums.iter().take(5) {
        println!(
            "album: {} — {} cover={:?}",
            album.artist, album.title, album.cover_id
        );
    }

    println!();
    println!("== artist image fetch ==");
    match image.fetch_cover_for_artist(&details.artist).await {
        Ok(cover) => {
            println!("request_key: {}", cover.request_key);
            println!("path: {:?}", cover.path);
            if cover.path.is_none() {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("artist image failed: {error}");
            std::process::exit(1);
        }
    }
}
