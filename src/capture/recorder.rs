use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::{Frame, FrameType, VideoFrame},
    Display, Target,
};

use super::window::WindowTarget;
use crate::error::{LoomError, Result};

pub struct RecorderConfig {
    pub target: WindowTarget,
    pub fps: u32,
    pub duration: Option<u32>,
}

pub struct Recorder {
    capturer: Capturer,
    config: RecorderConfig,
    running: Arc<AtomicBool>,
    frame_size: [u32; 2],
}

impl Recorder {
    pub fn new(config: RecorderConfig) -> Result<Self> {
        // WORKAROUND: scap's get_scale_factor() returns 0 for external windows
        // because it uses NSApp() which only works for windows owned by this process.
        // As a workaround, we capture the main display and crop to the window bounds.

        let main_display = scap::get_main_display();
        let display_target = Target::Display(Display {
            id: main_display.id,
            title: main_display.title,
            raw_handle: main_display.raw_handle,
        });

        // Crop to the window bounds to capture only the selected window
        let crop_area = Area {
            origin: Point {
                x: config.target.x as f64,
                y: config.target.y as f64,
            },
            size: Size {
                width: config.target.width as f64,
                height: config.target.height as f64,
            },
        };

        let options = Options {
            fps: config.fps,
            show_cursor: true,
            show_highlight: false,
            target: Some(display_target),
            crop_area: Some(crop_area),
            output_type: FrameType::BGRAFrame,
            excluded_targets: None,
            output_resolution: Default::default(),
            captures_audio: false,
            exclude_current_process_audio: false,
        };

        let capturer = Capturer::build(options)
            .map_err(|e| LoomError::CaptureFailed(e.to_string()))?;

        // Use dimensions from WindowTarget (the cropped area size)
        let frame_size = [config.target.width, config.target.height];

        let running = Arc::new(AtomicBool::new(false));

        Ok(Self {
            capturer,
            config,
            running,
            frame_size,
        })
    }

    /// Get the output frame size
    pub fn get_frame_size(&self) -> [u32; 2] {
        self.frame_size
    }

    /// Start recording, calling the callback for each captured frame
    pub fn start<F>(&mut self, mut on_frame: F) -> Result<()>
    where
        F: FnMut(&[u8], u32, u32) -> Result<()>,
    {
        self.running.store(true, Ordering::SeqCst);
        self.capturer.start_capture();

        let start_time = std::time::Instant::now();
        let duration_secs = self.config.duration.map(|d| d as u64);

        while self.running.load(Ordering::SeqCst) {
            // Check duration limit
            if let Some(secs) = duration_secs {
                if start_time.elapsed().as_secs() >= secs {
                    break;
                }
            }

            // Get next frame
            match self.capturer.get_next_frame() {
                Ok(Frame::Video(VideoFrame::BGRA(frame))) => {
                    on_frame(&frame.data, frame.width as u32, frame.height as u32)?;
                }
                Ok(other) => {
                    tracing::debug!("Received frame type: {:?}", std::mem::discriminant(&other));
                    continue;
                }
                Err(e) => {
                    tracing::warn!("Frame capture error: {}", e);
                    if !self.running.load(Ordering::SeqCst) {
                        break;
                    }
                    continue;
                }
            }
        }

        self.capturer.stop_capture();

        Ok(())
    }

    /// Get the running flag for external control (e.g., Ctrl+C)
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }
}
