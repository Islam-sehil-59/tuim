use std::{
    fs,
    io::{BufReader, Read},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Debug)]
pub struct CavaFrame {
    pub bars: Vec<u16>,
    pub max: u16,
}

pub struct CavaVisualizer {
    bars: usize,
    max: u16,
    child: Option<Child>,
    latest: Arc<Mutex<Vec<u16>>>,
    config_path: PathBuf,
    input_attempt: usize,
    unavailable: bool,
    last_error: Option<String>,
}

impl CavaVisualizer {
    pub fn new(bars: usize, max: u16) -> Self {
        Self {
            bars,
            max,
            child: None,
            latest: Arc::new(Mutex::new(Vec::new())),
            config_path: std::env::temp_dir()
                .join(format!("tuim-cava-{}.conf", std::process::id())),
            input_attempt: 0,
            unavailable: false,
            last_error: None,
        }
    }

    pub fn frame(&mut self, enabled: bool) -> Option<CavaFrame> {
        if !enabled {
            self.stop();
            self.input_attempt = 0;
            self.unavailable = false;
            return None;
        }

        self.reap_exited_child();
        if self.child.is_none() && !self.unavailable {
            self.start();
        }

        let bars = self.latest.lock().ok()?.clone();
        if bars.is_empty() {
            None
        } else {
            Some(CavaFrame {
                bars,
                max: self.max,
            })
        }
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn is_unavailable(&self) -> bool {
        self.unavailable
    }

    pub fn update_state(&mut self, state: &mut crate::state::visualizer::VisualizerState) {
        state.active = self.child.is_some();
        if let Some(error) = self.last_error.clone() {
            state.set_unavailable(error);
            return;
        }
        if self.unavailable {
            state.set_unavailable("cava visualizer is unavailable");
            return;
        }
        state.available = true;
        state.active = self.child.is_some();
        if state.last_error.is_some() {
            state.last_error = None;
        }
    }

    fn start(&mut self) {
        if let Err(error) = fs::write(&self.config_path, self.config()) {
            self.last_error = Some(format!("failed to write cava config: {error}"));
            self.unavailable = true;
            return;
        }

        let mut child = match Command::new("cava")
            .arg("-p")
            .arg(&self.config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                self.last_error = Some(format!("failed to spawn cava: {error}"));
                self.unavailable = true;
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let latest = Arc::clone(&self.latest);
            let bars = self.bars;
            thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut buffer = vec![0_u8; bars.saturating_mul(2)];
                while reader.read_exact(&mut buffer).is_ok() {
                    let bars = parse_cava_frame(&buffer);
                    if let Ok(mut current) = latest.lock() {
                        *current = bars;
                    }
                }
            });
        }

        self.child = Some(child);
    }

    fn reap_exited_child(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };

        match child.try_wait() {
            Ok(Some(_)) => {
                self.child = None;
                if let Ok(mut bars) = self.latest.lock() {
                    bars.clear();
                }
                if self.input_attempt + 1 < INPUT_METHODS.len() {
                    self.input_attempt += 1;
                } else {
                    self.last_error =
                        Some("cava exited: all input methods exhausted (pulse, pipewire)".into());
                    self.unavailable = true;
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.last_error = Some(format!("cava wait error: {error}"));
                self.child = None;
                self.unavailable = true;
            }
        }
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Ok(mut bars) = self.latest.lock() {
            bars.clear();
        }
    }

    pub fn reset(&mut self) {
        self.stop();
        self.input_attempt = 0;
        self.unavailable = false;
        self.last_error = None;
    }

    fn config(&self) -> String {
        let input = match INPUT_METHODS[self.input_attempt] {
            None => String::new(),
            Some(method) => format!(
                r#"
[input]
method = {method}
source = auto
"#
            ),
        };

        format!(
            r#"[general]
bars = {bars}
framerate = 60
autosens = 1
sensitivity = 130
{input}

[output]
method = raw
channels = mono
data_format = binary
bit_format = 16bit
reverse = 0

[smoothing]
noise_reduction = 25
monstercat = 0
waves = 0
"#,
            bars = self.bars,
            input = input,
        )
    }
}

impl Drop for CavaVisualizer {
    fn drop(&mut self) {
        self.stop();
        let _ = fs::remove_file(&self.config_path);
    }
}

fn parse_cava_frame(buffer: &[u8]) -> Vec<u16> {
    buffer
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}

const INPUT_METHODS: [Option<&str>; 3] = [None, Some("pulse"), Some("pipewire")];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_binary_cava_frames() {
        assert_eq!(
            parse_cava_frame(&[0, 0, 1, 0, 255, 0, 255, 255]),
            vec![0, 1, 255, 65535]
        );
    }
}
