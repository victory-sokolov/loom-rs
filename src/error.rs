use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoomError {
    #[error("No Chrome window found. Make sure Chrome is open with at least one window.")]
    WindowNotFound,

    #[error("ffmpeg not found. Install with: brew install ffmpeg")]
    FfmpegNotFound,

    #[error("Failed to start capture: {0}")]
    CaptureFailed(String),

    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),
}

pub type Result<T> = std::result::Result<T, LoomError>;
