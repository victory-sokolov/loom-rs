#[cfg(target_os = "macos")]
use crate::error::{LoomError, Result};
#[cfg(target_os = "macos")]
use scap::Target;

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct WindowTarget {
    pub title: String,
    #[allow(dead_code)]
    pub raw_target: Target, // Kept for future use when we implement proper window capture
    pub width: u32,
    pub height: u32,
    pub x: i32, // Window position on screen
    pub y: i32, // Window position on screen
}

/// List all windows available for capture
#[cfg(target_os = "macos")]
pub fn list_all_windows() -> Result<Vec<WindowTarget>> {
    let targets = scap::get_all_targets();

    let windows: Vec<WindowTarget> = targets
        .into_iter()
        .filter_map(|t| match &t {
            Target::Window(w) => {
                if w.title.is_empty() {
                    return None;
                }

                // Get window dimensions and position
                let (width, height, x, y) = get_window_dimensions(&t);

                Some(WindowTarget {
                    title: w.title.clone(),
                    raw_target: t,
                    width,
                    height,
                    x,
                    y,
                })
            }
            Target::Display(_) => None,
        })
        .collect();

    Ok(windows)
}

/// Get window dimensions and position using screencapturekit-sys directly
/// to access the full frame including origin
#[cfg(target_os = "macos")]
fn get_window_dimensions(target: &Target) -> (u32, u32, i32, i32) {
    use screencapturekit_sys::shareable_content::UnsafeSCShareableContent;

    match target {
        Target::Window(window) => {
            let content = match UnsafeSCShareableContent::get() {
                Ok(c) => c,
                Err(_) => return (1280, 720, 0, 0),
            };

            for w in content.windows() {
                if w.get_window_id() == window.id {
                    let frame = w.get_frame();
                    return (
                        frame.size.width as u32,
                        frame.size.height as u32,
                        frame.origin.x as i32,
                        frame.origin.y as i32,
                    );
                }
            }
            (1280, 720, 0, 0)
        }
        Target::Display(_display) => {
            let content = match UnsafeSCShareableContent::get() {
                Ok(c) => c,
                Err(_) => return (1920, 1080, 0, 0),
            };

            match content.displays().first() {
                Some(display) => {
                    let frame = display.get_frame();
                    (
                        frame.size.width as u32,
                        frame.size.height as u32,
                        frame.origin.x as i32,
                        frame.origin.y as i32,
                    )
                }
                None => (1920, 1080, 0, 0),
            }
        }
    }
}

/// Prompt user to select from multiple windows
#[cfg(target_os = "macos")]
pub fn prompt_window_selection(windows: &[WindowTarget]) -> Result<WindowTarget> {
    eprintln!("Available windows:");
    eprintln!();

    for (i, w) in windows.iter().enumerate() {
        eprintln!(
            "  [{}] {} ({}x{} at {},{})",
            i + 1,
            w.title,
            w.width,
            w.height,
            w.x,
            w.y
        );
    }

    eprintln!();
    eprint!("Select window [1-{}]: ", windows.len());

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|_| {
        LoomError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to read input",
        ))
    })?;

    let selection: usize = input
        .trim()
        .parse()
        .map_err(|_| LoomError::WindowNotFound)?;

    if selection == 0 || selection > windows.len() {
        return Err(LoomError::WindowNotFound);
    }

    Ok(windows[selection - 1].clone())
}
