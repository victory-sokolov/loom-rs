#[cfg(target_os = "macos")]
mod recorder;
#[cfg(target_os = "macos")]
mod window;

#[cfg(target_os = "macos")]
pub use recorder::{Recorder, RecorderConfig};
#[cfg(target_os = "macos")]
pub use window::{list_all_windows, prompt_window_selection};

// Non-macOS stubs - this tool is macOS-only
#[cfg(not(target_os = "macos"))]
pub struct Recorder;
#[cfg(not(target_os = "macos"))]
pub struct RecorderConfig;
#[cfg(not(target_os = "macos"))]
pub struct WindowTarget {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}
