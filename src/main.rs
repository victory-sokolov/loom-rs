mod capture;
mod cli;
mod encode;
mod error;
mod output;

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Instant;

use capture::{list_all_windows, prompt_window_selection, Recorder, RecorderConfig};
use cli::Args;
use encode::{EncodeConfig, FfmpegEncoder};
use error::Result;
use output::{copy_path_to_clipboard, generate_llm_prompt};
use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let args = Args::parse_args();

    // Setup logging
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy(),
        )
        .init();

    // Check ffmpeg availability early
    FfmpegEncoder::check_available()?;

    // Check screen capture permission
    if !scap::has_permission() {
        eprintln!("Screen capture permission not granted.");
        eprintln!(
            "Please grant permission in System Settings > Privacy & Security > Screen Recording"
        );
        scap::request_permission();
        return Ok(());
    }

    // List mode - just show windows and exit
    if args.list {
        return list_mode();
    }

    // Get resolution before any moves
    let target_resolution = args.resolution_tuple();

    // Find target window
    let windows = list_all_windows()?;

    let target = match windows.len() {
        0 => {
            eprintln!("No windows found. Open a window and try again.");
            return Ok(());
        }
        _ => {
            if let Some(filter) = &args.target {
                let matches: Vec<_> = windows
                    .iter()
                    .filter(|w| w.title.to_lowercase().contains(&filter.to_lowercase()))
                    .collect();

                match matches.as_slice() {
                    [] => {
                        eprintln!("No window matching '{}' found.", filter);
                        return Ok(());
                    }
                    [one] => (*one).clone(),
                    _ => {
                        let owned: Vec<_> = matches.into_iter().cloned().collect();
                        prompt_window_selection(&owned)?
                    }
                }
            } else {
                prompt_window_selection(&windows)?
            }
        }
    };

    tracing::info!("Recording: {}", target.title);

    // Prepare output path
    let output_path = args.output.as_ref().map(PathBuf::from).unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let dir = PathBuf::from(home).join("vloom-recordings");
        std::fs::create_dir_all(&dir).ok();
        let timestamp = chrono_timestamp();
        dir.join(format!("vloom-{}.mp4", timestamp))
    });

    // Create recorder first to get actual frame dimensions
    let recorder_config = RecorderConfig {
        target: target.clone(),
        fps: args.fps,
        duration: args.duration,
    };

    let mut recorder = Recorder::new(recorder_config)?;
    let running = recorder.running_flag();

    // Get actual frame dimensions from capturer
    let frame_size = recorder.get_frame_size();
    let (width, height) = (frame_size[0], frame_size[1]);

    tracing::info!("Frame size: {}x{}", width, height);

    // Create encoder with actual dimensions
    // Use CRF 18 for better screen recording quality (default 23 is too lossy)
    let encode_config = EncodeConfig {
        width,
        height,
        fps: args.fps,
        output_path: output_path.clone(),
        target_resolution,
        crf: 18,
    };

    let mut encoder = FfmpegEncoder::spawn(encode_config)?;
    tracing::debug!("FFmpeg encoder started");

    // Setup Ctrl+C handler
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        eprintln!("\nStopping recording...");
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Start recording
    let start_time = Instant::now();
    let frame_count = std::cell::Cell::new(0u64);

    eprintln!("Recording: {}", target.title);
    eprintln!("Press Ctrl+C to stop.");
    if let Some(duration) = args.duration {
        eprintln!("Auto-stopping in {} seconds.", duration);
    }

    recorder.start(|frame_data, _width, _height| {
        encoder.write_frame(frame_data)?;
        frame_count.set(frame_count.get() + 1);

        // Progress indicator every 30 frames (~1 second)
        if frame_count.get() % 30 == 0 {
            let elapsed = start_time.elapsed().as_secs();
            eprint!("\rRecording... {}s elapsed", elapsed);
        }

        Ok(())
    })?;

    // Finalize encoding
    eprintln!("\nFinalizing video...");
    let final_path = encoder.finalize()?;

    // Report results
    let elapsed = start_time.elapsed();
    let metadata = std::fs::metadata(&final_path)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

    eprintln!();
    eprintln!("Recording saved: {}", final_path.display());
    eprintln!("Duration: {:.1}s", elapsed.as_secs_f64());
    eprintln!("Size: {:.2} MB", size_mb);

    // Copy to clipboard
    copy_path_to_clipboard(&final_path)?;
    eprintln!("Path copied to clipboard!");

    // Show LLM prompt
    eprintln!();
    eprintln!("LLM Prompt:");
    eprintln!("────────────────────────────────────────");
    println!("{}", generate_llm_prompt(&final_path));
    eprintln!("────────────────────────────────────────");

    Ok(())
}

fn list_mode() -> Result<()> {
    let windows = list_all_windows()?;

    if windows.is_empty() {
        eprintln!("No capturable windows found.");
        return Ok(());
    }

    eprintln!("Available windows:");
    eprintln!();

    for (i, w) in windows.iter().enumerate() {
        eprintln!("  [{}] {}", i + 1, w.title);
    }

    eprintln!();
    eprintln!("Use --target \"<title>\" to select a specific window.");

    Ok(())
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let total_secs = now.as_secs();
    let days = total_secs / 86400;
    let secs_remaining = total_secs % 86400;
    let hours = secs_remaining / 3600;
    let minutes = (secs_remaining % 3600) / 60;
    let seconds = secs_remaining % 60;

    // Approximate date from Unix epoch (1970-01-01)
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!(
        "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
        year, month, day, hours, minutes, seconds
    )
}
