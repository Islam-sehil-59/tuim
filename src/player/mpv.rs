use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

use crate::{
    log::player_log,
    media::playback::PlaybackSource,
    player::{error::PlayerError, traits::Player},
};

const IPC_TIMEOUT: Duration = Duration::from_millis(200);

struct ActivePlayback {
    child: Option<Child>,
    source: String,
    ipc_path: PathBuf,
}

pub struct MpvPlayer {
    active: Option<ActivePlayback>,
}

pub struct PlaybackExit {
    pub message: String,
    pub success: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PlaybackProgress {
    pub position_secs: f64,
    pub duration_secs: Option<f64>,
    pub audio_codec: Option<String>,
    pub audio_bitrate_kbps: Option<u64>,
    pub volume: Option<u8>,
}

impl PlaybackProgress {
    pub fn ratio(&self) -> f64 {
        let Some(duration) = self.duration_secs else {
            return 0.0;
        };

        if duration <= 0.0 {
            return 0.0;
        }

        (self.position_secs / duration).clamp(0.0, 1.0)
    }

    pub fn label(&self) -> String {
        let position = format_timestamp(self.position_secs);
        let duration = self
            .duration_secs
            .map(format_timestamp)
            .unwrap_or_else(|| String::from("--:--"));

        format!("{position} / {duration}")
    }
}

impl MpvPlayer {
    pub fn new() -> Self {
        player_log("player initialized");
        let ipc_path = mpv_ipc_path();
        let active = if connect_ipc(&ipc_path).is_ok() {
            player_log(&format!(
                "reattached to existing mpv ipc={}",
                ipc_path.display()
            ));
            Some(ActivePlayback {
                child: None,
                source: String::from("attached external playback"),
                ipc_path,
            })
        } else {
            None
        };

        Self { active }
    }

    pub fn is_running(&mut self) -> bool {
        if self.poll_exit().is_some() {
            return false;
        }

        self.active.is_some()
    }

    pub fn poll_exit(&mut self) -> Option<PlaybackExit> {
        let Some(active) = self.active.as_mut() else {
            return None;
        };

        let Some(child) = active.child.as_mut() else {
            if connect_ipc(&active.ipc_path).is_ok() {
                return None;
            }

            let ipc_path = active.ipc_path.clone();
            player_log(&format!(
                "attached mpv ipc disappeared path={}",
                ipc_path.display()
            ));
            self.active = None;
            cleanup_ipc_socket(&ipc_path);
            return Some(PlaybackExit {
                message: String::from("Playback ended (external mpv socket closed)."),
                success: true,
            });
        };

        match child.try_wait() {
            Ok(Some(status)) => {
                let pid = child.id();
                let source = active.source.clone();
                let ipc_path = active.ipc_path.clone();
                player_log(&format!(
                    "mpv exited pid={pid} status={status} source={source}"
                ));
                self.active = None;
                cleanup_ipc_socket(&ipc_path);

                Some(if status.success() {
                    PlaybackExit {
                        message: String::from("Playback ended (mpv reached end of stream)."),
                        success: true,
                    }
                } else {
                    PlaybackExit {
                        message: format!("Playback stopped (mpv exit: {status})."),
                        success: false,
                    }
                })
            }
            Ok(None) => None,
            Err(error) => {
                player_log(&format!("mpv try_wait failed: {error}"));
                let ipc_path = active.ipc_path.clone();
                self.active = None;
                cleanup_ipc_socket(&ipc_path);
                Some(PlaybackExit {
                    message: format!("Playback state check failed: {error}"),
                    success: false,
                })
            }
        }
    }

    pub fn poll_progress(&self) -> Option<PlaybackProgress> {
        let active = self.active.as_ref()?;
        let position_secs = query_numeric_property(&active.ipc_path, "time-pos")?;
        let duration_secs = query_numeric_property(&active.ipc_path, "duration");
        let audio_codec = query_string_property(&active.ipc_path, "audio-codec-name");
        let audio_bitrate_kbps = query_numeric_property(&active.ipc_path, "audio-bitrate")
            .map(|bitrate| (bitrate / 1000.0).round().max(0.0) as u64);
        let volume =
            query_numeric_property(&active.ipc_path, "volume").map(|volume| volume.round() as u8);

        Some(PlaybackProgress {
            position_secs,
            duration_secs,
            audio_codec,
            audio_bitrate_kbps,
            volume,
        })
    }

    pub fn toggle_pause(&self) -> Result<(), PlayerError> {
        let Some(active) = self.active.as_ref() else {
            return Err(PlayerError::NoActivePlayback);
        };

        send_mpv_command(&active.ipc_path, r#"{"command":["cycle","pause"]}"#)
    }

    pub fn seek_relative(&self, seconds: i64) -> Result<(), PlayerError> {
        let Some(active) = self.active.as_ref() else {
            return Err(PlayerError::NoActivePlayback);
        };

        send_mpv_command(
            &active.ipc_path,
            &format!(r#"{{"command":["seek",{seconds},"relative"]}}"#),
        )
    }

    pub fn change_volume(&self, delta: i8) -> Result<(), PlayerError> {
        let Some(active) = self.active.as_ref() else {
            return Err(PlayerError::NoActivePlayback);
        };

        send_mpv_command(
            &active.ipc_path,
            &format!(r#"{{"command":["add","volume",{delta}]}}"#),
        )
    }

    pub fn toggle_mute(&self) -> Result<(), PlayerError> {
        let Some(active) = self.active.as_ref() else {
            return Err(PlayerError::NoActivePlayback);
        };

        send_mpv_command(&active.ipc_path, r#"{"command":["cycle","mute"]}"#)
    }

    pub fn is_paused(&self) -> Option<bool> {
        let active = self.active.as_ref()?;
        query_bool_property(&active.ipc_path, "pause")
    }

    pub fn stop(&mut self) -> bool {
        if self.active.is_none() {
            return false;
        }

        self.stop_current("stopped by user");
        true
    }

    fn stop_current(&mut self, reason: &str) {
        let Some(mut active) = self.active.take() else {
            return;
        };

        let pid = active
            .child
            .as_ref()
            .map(|child| child.id().to_string())
            .unwrap_or_else(|| String::from("attached"));
        player_log(&format!(
            "stopping current playback pid={pid} reason={reason} source={}",
            active.source
        ));

        if let Some(mut child) = active.child.take() {
            if let Err(error) = child.kill() {
                player_log(&format!("failed to kill mpv pid={pid}: {error}"));
            }

            match child.wait() {
                Ok(status) => player_log(&format!("mpv wait after stop pid={pid} status={status}")),
                Err(error) => player_log(&format!("mpv wait failed pid={pid}: {error}")),
            }
        } else if let Err(error) = send_mpv_command(&active.ipc_path, r#"{"command":["quit"]}"#) {
            player_log(&format!("failed to quit attached mpv: {error}"));
        }

        cleanup_ipc_socket(&active.ipc_path);
    }

    pub fn shutdown_for_app_exit(&mut self, stop_if_paused: bool) {
        if stop_if_paused && self.is_paused().unwrap_or(false) {
            self.stop_current("app closed while paused");
            return;
        }

        if let Some(active) = &self.active {
            player_log(&format!(
                "detaching from mpv on app exit source={} ipc={}",
                active.source,
                active.ipc_path.display()
            ));
        }
        self.active = None;
    }
}

impl Player for MpvPlayer {
    fn play(&mut self, source: &PlaybackSource) -> Result<(), PlayerError> {
        if self.active.is_some() {
            player_log("new playback request arrived while another track is active");
        }
        self.stop_current("replaced by new playback request");

        let mpv_log = mpv_log_path();
        let ipc_path = mpv_ipc_path();
        cleanup_ipc_socket(&ipc_path);
        let stream_url = &source.url;
        let header_args = source
            .headers
            .iter()
            .map(|(name, value)| format!("--http-header-fields={name}: {value}"))
            .collect::<Vec<_>>();
        let header_preview = header_args.join(" ");
        let command_preview = format!(
            "mpv --no-video --force-window=no --really-quiet --msg-level=all=info --log-file={} --input-ipc-server={} {} --demuxer-lavf-o=protocol_whitelist=[file,http,https,tcp,tls,crypto] {}",
            mpv_log.display(),
            ipc_path.display(),
            header_preview,
            stream_url
        );
        player_log(&format!("spawning mpv command={command_preview}"));
        player_log(&format!("playback source={stream_url}"));

        let mut command = Command::new("mpv");
        command
            .arg("--no-video")
            .arg("--force-window=no")
            .arg("--really-quiet")
            .arg("--msg-level=all=info")
            .arg(format!("--log-file={}", mpv_log.display()))
            .arg(format!("--input-ipc-server={}", ipc_path.display()));
        for header in header_args {
            command.arg(header);
        }
        let child = command
            .arg("--demuxer-lavf-o=protocol_whitelist=[file,http,https,tcp,tls,crypto]")
            .arg(stream_url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                player_log(&format!("failed to spawn mpv: {error}"));
                PlayerError::Spawn(error.to_string())
            })?;

        let pid = child.id();
        player_log(&format!("mpv spawned pid={pid}"));
        self.active = Some(ActivePlayback {
            child: Some(child),
            source: stream_url.to_string(),
            ipc_path,
        });
        Ok(())
    }
}

fn mpv_log_path() -> PathBuf {
    crate::log::log_path("mpv.log")
}

fn mpv_ipc_path() -> PathBuf {
    std::env::temp_dir().join("tuim-mpv.sock")
}

fn cleanup_ipc_socket(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        player_log(&format!(
            "failed to remove mpv ipc socket path={} error={error}",
            path.display()
        ));
    }
}

fn query_numeric_property(ipc_path: &Path, property: &str) -> Option<f64> {
    let stream = UnixStream::connect(ipc_path).ok()?;
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .ok()?;
    stream
        .set_write_timeout(Some(IPC_TIMEOUT))
        .ok()?;

    let mut stream = stream;
    let command = format!(r#"{{"command":["get_property","{property}"]}}"#);
    stream.write_all(command.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;

    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut response).ok()?;

    let value: serde_json::Value = serde_json::from_str(response.trim()).ok()?;
    if value.get("error").and_then(serde_json::Value::as_str) != Some("success") {
        return None;
    }

    value.get("data").and_then(serde_json::Value::as_f64)
}

fn query_string_property(ipc_path: &Path, property: &str) -> Option<String> {
    let value = query_property(ipc_path, property)?;
    value.as_str().map(str::to_string)
}

fn query_bool_property(ipc_path: &Path, property: &str) -> Option<bool> {
    let value = query_property(ipc_path, property)?;
    value.as_bool()
}

fn query_property(ipc_path: &Path, property: &str) -> Option<serde_json::Value> {
    let stream = UnixStream::connect(ipc_path).ok()?;
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .ok()?;
    stream
        .set_write_timeout(Some(IPC_TIMEOUT))
        .ok()?;
    let mut stream = stream;
    let command = format!(r#"{{"command":["get_property","{property}"]}}"#);
    stream.write_all(command.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;

    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut response).ok()?;

    let value: serde_json::Value = serde_json::from_str(response.trim()).ok()?;
    if value.get("error").and_then(serde_json::Value::as_str) != Some("success") {
        return None;
    }

    value.get("data").cloned()
}

fn send_mpv_command(ipc_path: &Path, command: &str) -> Result<(), PlayerError> {
    let mut stream = connect_ipc(ipc_path).map_err(|error| PlayerError::Ipc(error.to_string()))?;
    stream
        .write_all(command.as_bytes())
        .map_err(|error| PlayerError::Ipc(error.to_string()))?;
    stream
        .write_all(b"\n")
        .map_err(|error| PlayerError::Ipc(error.to_string()))
}

fn connect_ipc(ipc_path: &Path) -> std::io::Result<UnixStream> {
    let stream = UnixStream::connect(ipc_path)?;
    stream.set_read_timeout(Some(IPC_TIMEOUT))?;
    stream.set_write_timeout(Some(IPC_TIMEOUT))?;
    Ok(stream)
}

fn format_timestamp(seconds: f64) -> String {
    let seconds = seconds.max(0.0).round() as u64;
    let minutes = seconds / 60;
    let seconds = seconds % 60;

    format!("{minutes}:{seconds:02}")
}


