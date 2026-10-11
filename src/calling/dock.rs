//! Docking: during a call, the call window takes the left part of where Wren
//! was and Wren becomes a narrow panel on the right, with the meeting chat.
//!
//! Everything here is in screen points (what browsers call DIPs). Wren moves
//! only its own window; the browser places the call window from
//! `--window-position` and `--window-size`. Wayland lets no app choose its
//! position, so there Wren cannot know its frame and does not dock.

use super::browser::Placement;

/// Wren's width while docked.
pub const PANEL_WIDTH: f32 = 380.0;
/// Below this frame width the call window would be too small to share.
pub const MIN_FRAME_WIDTH: f32 = 960.0;
const MIN_FRAME_HEIGHT: f32 = 400.0;
/// Space between the call window and Wren.
const GAP: f32 = 8.0;

/// A window's outer rectangle in screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    /// Where the browser opens the call window.
    pub call: Placement,
    /// Wren's own outer frame while docked.
    pub wren: Frame,
}

/// Splits Wren's current frame between the call and Wren, or `None` when it
/// is too small to share.
pub fn plan(frame: Frame) -> Option<Plan> {
    if frame.width < MIN_FRAME_WIDTH || frame.height < MIN_FRAME_HEIGHT {
        return None;
    }
    let call_width = frame.width - PANEL_WIDTH - GAP;
    Some(Plan {
        call: Placement {
            x: frame.x.round() as i32,
            y: frame.y.round() as i32,
            width: call_width.round() as i32,
            height: frame.height.round() as i32,
        },
        wren: Frame {
            x: frame.x + frame.width - PANEL_WIDTH,
            y: frame.y,
            width: PANEL_WIDTH,
            height: frame.height,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_frame_is_split_with_wren_on_the_right() {
        let plan = plan(Frame {
            x: 100.0,
            y: 50.0,
            width: 1240.0,
            height: 800.0,
        })
        .unwrap();
        assert_eq!(
            plan.call,
            Placement {
                x: 100,
                y: 50,
                width: 852,
                height: 800
            }
        );
        assert_eq!(plan.wren.x, 960.0);
        assert_eq!(plan.wren.width, PANEL_WIDTH);
        // Nothing overlaps.
        assert!((plan.call.x + plan.call.width) as f32 <= plan.wren.x);
    }

    #[test]
    fn small_frames_are_not_split() {
        let frame = Frame {
            x: 0.0,
            y: 0.0,
            width: 900.0,
            height: 800.0,
        };
        assert!(plan(frame).is_none());
        assert!(
            plan(Frame {
                width: 1400.0,
                height: 300.0,
                ..frame
            })
            .is_none()
        );
    }
}
