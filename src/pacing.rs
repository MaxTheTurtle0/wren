//! A ceiling for native render loops whose OpenGL swap does not block for vsync.
//! This never schedules a frame; settled and hidden windows still sleep.

use std::time::{Duration, Instant};

pub struct FramePacer {
    interval: Duration,
    last: Option<Instant>,
    checked: Instant,
}

impl FramePacer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let millihertz = cc
            .winit_window()
            .and_then(|w| w.current_monitor())
            .and_then(|m| m.refresh_rate_millihertz())
            .unwrap_or(60_000);
        Self {
            interval: Duration::from_secs_f64(
                1000.0 / f64::from(millihertz.clamp(30_000, 240_000)),
            ),
            last: None,
            checked: Instant::now(),
        }
    }

    /// Like a blocking vsync swap, wait only for the remainder of this frame.
    /// A late frame starts immediately; input never queues catch-up frames.
    pub fn begin_frame(&mut self, frame: &eframe::Frame) {
        // Recheck only while frames are already happening. Moving between
        // monitors must not leave a high-refresh display capped to the old one.
        if self.checked.elapsed() >= Duration::from_secs(1) {
            if let Some(rate) = frame
                .winit_window()
                .and_then(|w| w.current_monitor())
                .and_then(|m| m.refresh_rate_millihertz())
            {
                self.interval =
                    Duration::from_secs_f64(1000.0 / f64::from(rate.clamp(30_000, 240_000)));
            }
            self.checked = Instant::now();
        }
        if let Some(last) = self.last {
            let remaining = self.interval.saturating_sub(last.elapsed());
            if !remaining.is_zero() {
                std::thread::sleep(remaining);
            }
        }
        self.last = Some(Instant::now());
    }
}
