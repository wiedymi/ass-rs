//! Drawing command processing module

mod parse;
mod spline;

pub use parse::parse_draw_commands;
use spline::spline_to_bezier;

use crate::utils::RenderError;
use tiny_skia::{Path, PathBuilder};

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// Drawing command types
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Move to position (m command)
    MoveTo { x: f32, y: f32 },
    /// Move without drawing (n command)
    MoveToNoDraw { x: f32, y: f32 },
    /// Line to position (l command)
    LineTo { x: f32, y: f32 },
    /// Cubic Bezier curve (b command)
    BezierTo {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
    /// B-spline (s command - converted to bezier internally)
    Spline { points: Vec<(f32, f32)> },
    /// Extended B-spline (p command)
    ExtendSpline { points: Vec<(f32, f32)> },
    /// Close path (c command)
    ClosePath,
}

/// Process ASS drawing commands into a path
pub fn process_drawing_commands(commands: &str) -> Result<Option<Path>, RenderError> {
    // Try to parse the commands, return None for invalid input
    let draw_commands = match parse_draw_commands(commands) {
        Ok(commands) => commands,
        Err(_) => return Ok(None), // Invalid drawing commands return None
    };
    if draw_commands.is_empty() {
        return Ok(None);
    }

    let mut builder = PathBuilder::new();
    let mut _current_pos = (0.0, 0.0);

    for cmd in draw_commands {
        match cmd {
            DrawCommand::MoveTo { x, y } => {
                builder.move_to(x, y);
                _current_pos = (x, y);
            }
            DrawCommand::MoveToNoDraw { x, y } => {
                builder.move_to(x, y);
                _current_pos = (x, y);
            }
            DrawCommand::LineTo { x, y } => {
                builder.line_to(x, y);
                _current_pos = (x, y);
            }
            DrawCommand::BezierTo {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                builder.cubic_to(x1, y1, x2, y2, x3, y3);
                _current_pos = (x3, y3);
            }
            DrawCommand::Spline { ref points } => {
                // Convert B-spline to Bezier curves
                if points.len() >= 3 {
                    let beziers = spline_to_bezier(points, false);
                    for (c1, c2, end) in beziers {
                        builder.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
                        _current_pos = end;
                    }
                }
            }
            DrawCommand::ExtendSpline { ref points } => {
                // Extended B-spline with additional control
                if points.len() >= 3 {
                    let beziers = spline_to_bezier(points, true);
                    for (c1, c2, end) in beziers {
                        builder.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
                        _current_pos = end;
                    }
                }
            }
            DrawCommand::ClosePath => {
                builder.close();
            }
        }
    }

    Ok(builder.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_drawing_parses() {
        let path = process_drawing_commands("m 0 0 l 100 0 l 100 100 l 0 100")
            .expect("ok")
            .expect("some path");
        let b = path.bounds();
        assert!(b.width() > 50.0 && b.height() > 50.0);
    }

    #[test]
    fn trailing_tag_does_not_discard_the_shape() {
        // Real scripts append the drawing's closing tag to the `\p` text, e.g.
        // `...l 0 100\p0}`. The backslash must end the drawing, not poison the
        // last coordinate and throw the whole shape away (which rendered nothing
        // for every elaborate sign — the gradient boxes, brushstrokes, etc.).
        let path = process_drawing_commands("m 0 0 l 100 0 l 100 100 l 0 100\\p0}")
            .expect("ok")
            .expect("some path");
        let b = path.bounds();
        assert!(
            b.width() > 50.0 && b.height() > 50.0,
            "shape with a trailing \\p0 tag was discarded: bounds {b:?}"
        );

        // The same shape with and without the trailing tag must be identical.
        let clean = process_drawing_commands("m 0 0 l 100 0 l 100 100 l 0 100")
            .expect("ok")
            .expect("some");
        assert_eq!(path.len(), clean.len());
    }
}
