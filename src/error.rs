use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoomError {
    #[cfg(target_os = "macos")]
    #[error("No Chrome window found. Make sure Chrome is open with at least one window.")]
    WindowNotFound,

    #[cfg(target_os = "macos")]
    #[error("ffmpeg not found. Install with: brew install ffmpeg")]
    FfmpegNotFound,

    #[cfg(target_os = "macos")]
    #[error("Failed to start capture: {0}")]
    CaptureFailed(String),

    #[cfg(target_os = "macos")]
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(target_os = "macos")]
    #[error("Clipboard error: {0}")]
    Clipboard(String),
}

pub type Result<T> = std::result::Result<T, LoomError>;
