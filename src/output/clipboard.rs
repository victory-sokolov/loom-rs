use std::path::Path;

use crate::error::{LoomError, Result};

/// Copy text to system clipboard
pub fn copy_path_to_clipboard(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| LoomError::Clipboard(e.to_string()))?;

    clipboard
        .set_text(path_str.as_ref())
        .map_err(|e| LoomError::Clipboard(e.to_string()))?;

    Ok(())
}

/// Generate an LLM-ready prompt for the recording
pub fn generate_llm_prompt(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    format!(
        r#"I recorded a video of a feature implementation. Please analyze this recording:

Video path: {}

The video shows my web application running in Chrome. Please:
1. Describe what you observe in the UI
2. Identify any potential UX improvements
3. Note any bugs or unexpected behavior

[Attach the video file to your message]
"#,
        path_str
    )
}
