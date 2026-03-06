use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};

use crate::error::{LoomError, Result};

pub struct EncodeConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub output_path: PathBuf,
    pub target_resolution: (u32, u32),
    pub crf: u8,
}

pub struct FfmpegEncoder {
    process: Child,
    stdin: ChildStdin,
    output_path: PathBuf,
}

impl FfmpegEncoder {
    /// Check if ffmpeg is available
    pub fn check_available() -> Result<()> {
        which::which("ffmpeg")
            .map_err(|_| LoomError::FfmpegNotFound)?;
        Ok(())
    }

    /// Spawn ffmpeg encoder process
    pub fn spawn(config: EncodeConfig) -> Result<Self> {
        Self::check_available()?;

        let (target_w, target_h) = config.target_resolution;
        let scale_filter = format!(
            "scale={target_w}:{target_h}:force_original_aspect_ratio=decrease,pad={target_w}:{target_h}:(ow-iw)/2:(oh-ih)/2"
        );

        let output_str = config.output_path.to_string_lossy().into_owned();

        let mut cmd = Command::new("ffmpeg");
        cmd.args([
            "-y",  // Overwrite output
            "-f", "rawvideo",
            "-pixel_format", "bgra",
            "-video_size", &format!("{}x{}", config.width, config.height),
            "-framerate", &format!("{}", config.fps),
            "-i", "-",  // Read from stdin
            "-c:v", "libx264",
            "-preset", "fast",
            "-crf", &format!("{}", config.crf),
            "-pix_fmt", "yuv420p",
            "-vf", &scale_filter,
            "-movflags", "+faststart",  // Web-friendly output
        ])
        .arg(&output_str)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

        tracing::debug!("Running: {:?}", cmd);

        let mut process = cmd
            .spawn()
            .map_err(|e| LoomError::EncodingFailed(e.to_string()))?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| LoomError::EncodingFailed("Failed to open ffmpeg stdin".into()))?;

        Ok(Self {
            process,
            stdin,
            output_path: config.output_path,
        })
    }

    /// Write a frame to the encoder
    pub fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        self.stdin
            .write_all(frame)
            .map_err(|e| LoomError::EncodingFailed(e.to_string()))
    }

    /// Finalize encoding and wait for ffmpeg to finish
    pub fn finalize(mut self) -> Result<PathBuf> {
        // Close stdin to signal end of input
        drop(self.stdin);

        // Wait for ffmpeg to finish
        let status = self
            .process
            .wait()
            .map_err(|e| LoomError::EncodingFailed(e.to_string()))?;

        if !status.success() {
            return Err(LoomError::EncodingFailed(format!(
                "ffmpeg exited with status: {}",
                status
            )));
        }

        Ok(self.output_path)
    }
}
