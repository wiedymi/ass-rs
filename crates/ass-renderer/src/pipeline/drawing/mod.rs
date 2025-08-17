//! Drawing command processing module

#[cfg(feature = "nostd")]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(not(feature = "nostd"))]
use std::{
    string::{String, ToString},
    vec::Vec,
};

use crate::utils::RenderError;
use tiny_skia::{Path, PathBuilder};

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
    let mut current_pos = (0.0, 0.0);

    for cmd in draw_commands {
        match cmd {
            DrawCommand::MoveTo { x, y } => {
                builder.move_to(x, y);
                current_pos = (x, y);
            }
            DrawCommand::MoveToNoDraw { x, y } => {
                builder.move_to(x, y);
                current_pos = (x, y);
            }
            DrawCommand::LineTo { x, y } => {
                builder.line_to(x, y);
                current_pos = (x, y);
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
                current_pos = (x3, y3);
            }
            DrawCommand::Spline { ref points } => {
                // Convert B-spline to Bezier curves
                if points.len() >= 3 {
                    let beziers = spline_to_bezier(points, false);
                    for (c1, c2, end) in beziers {
                        builder.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
                        current_pos = end;
                    }
                }
            }
            DrawCommand::ExtendSpline { ref points } => {
                // Extended B-spline with additional control
                if points.len() >= 3 {
                    let beziers = spline_to_bezier(points, true);
                    for (c1, c2, end) in beziers {
                        builder.cubic_to(c1.0, c1.1, c2.0, c2.1, end.0, end.1);
                        current_pos = end;
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

/// Parse drawing commands from string
pub fn parse_draw_commands(text: &str) -> Result<Vec<DrawCommand>, RenderError> {
    let mut commands = Vec::new();
    let mut tokens = tokenize_drawing_commands(text);
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].as_str() {
            "m" => {
                // Move to - can have multiple coordinate pairs
                i += 1;
                if i + 1 < tokens.len() {
                    // First move is always MoveTo
                    let x = parse_coord(&tokens[i])?;
                    let y = parse_coord(&tokens[i + 1])?;
                    commands.push(DrawCommand::MoveTo { x, y });
                    i += 2;

                    // Additional coordinate pairs become LineTo
                    while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                        let x = parse_coord(&tokens[i])?;
                        let y = parse_coord(&tokens[i + 1])?;
                        commands.push(DrawCommand::LineTo { x, y });
                        i += 2;
                    }
                } else {
                    return Err(RenderError::InvalidDrawCommand(
                        "Incomplete move command".to_string(),
                    ));
                }
            }
            "n" => {
                // Move to without drawing - can have multiple coordinate pairs
                i += 1;
                while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                    let x = parse_coord(&tokens[i])?;
                    let y = parse_coord(&tokens[i + 1])?;
                    commands.push(DrawCommand::MoveToNoDraw { x, y });
                    i += 2;
                }

                if commands.is_empty()
                    || !matches!(commands.last(), Some(DrawCommand::MoveToNoDraw { .. }))
                {
                    return Err(RenderError::InvalidDrawCommand(
                        "Incomplete move command".to_string(),
                    ));
                }
            }
            "l" => {
                // Line to - can have multiple coordinate pairs
                i += 1;
                while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                    let x = parse_coord(&tokens[i])?;
                    let y = parse_coord(&tokens[i + 1])?;
                    commands.push(DrawCommand::LineTo { x, y });
                    i += 2;
                }

                if commands.is_empty()
                    || !matches!(commands.last(), Some(DrawCommand::LineTo { .. }))
                {
                    return Err(RenderError::InvalidDrawCommand(
                        "Incomplete line command".to_string(),
                    ));
                }
            }
            "b" => {
                // Bezier curve - can have multiple sets of 6 coordinates
                i += 1;
                while i + 5 < tokens.len() && !is_command(&tokens[i]) {
                    let x1 = parse_coord(&tokens[i])?;
                    let y1 = parse_coord(&tokens[i + 1])?;
                    let x2 = parse_coord(&tokens[i + 2])?;
                    let y2 = parse_coord(&tokens[i + 3])?;
                    let x3 = parse_coord(&tokens[i + 4])?;
                    let y3 = parse_coord(&tokens[i + 5])?;
                    commands.push(DrawCommand::BezierTo {
                        x1,
                        y1,
                        x2,
                        y2,
                        x3,
                        y3,
                    });
                    i += 6;
                }

                if commands.is_empty()
                    || !matches!(commands.last(), Some(DrawCommand::BezierTo { .. }))
                {
                    return Err(RenderError::InvalidDrawCommand(
                        "Incomplete bezier command".to_string(),
                    ));
                }
            }
            "s" => {
                // B-spline (at least 3 points)
                let mut points = Vec::new();
                i += 1;

                // Collect points until we hit another command or end
                while i + 1 < tokens.len() {
                    if is_command(&tokens[i]) {
                        break;
                    }
                    let x = parse_coord(&tokens[i])?;
                    let y = parse_coord(&tokens[i + 1])?;
                    points.push((x, y));
                    i += 2;
                }

                if points.len() >= 3 {
                    // Close the spline by repeating first point if needed
                    let first = points[0];
                    let last = points[points.len() - 1];
                    if (first.0 - last.0).abs() > 0.01 || (first.1 - last.1).abs() > 0.01 {
                        points.push(first);
                    }
                    commands.push(DrawCommand::Spline { points });
                } else if !points.is_empty() {
                    return Err(RenderError::InvalidDrawCommand(
                        "B-spline needs at least 3 points".to_string(),
                    ));
                }
            }
            "p" => {
                // Extended B-spline
                let mut points = Vec::new();
                i += 1;

                // Collect points for extended spline
                while i + 1 < tokens.len() {
                    if is_command(&tokens[i]) {
                        break;
                    }
                    let x = parse_coord(&tokens[i])?;
                    let y = parse_coord(&tokens[i + 1])?;
                    points.push((x, y));
                    i += 2;
                }

                if points.len() >= 3 {
                    commands.push(DrawCommand::ExtendSpline { points });
                } else if !points.is_empty() {
                    return Err(RenderError::InvalidDrawCommand(
                        "Extended spline needs at least 3 points".to_string(),
                    ));
                }
            }
            "c" => {
                // Close path
                commands.push(DrawCommand::ClosePath);
                i += 1;
            }
            _ => {
                // Unknown command or coordinate without command
                i += 1;
            }
        }
    }

    Ok(commands)
}

/// Tokenize drawing command string
fn tokenize_drawing_commands(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if ch.is_ascii_alphabetic() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            tokens.push(ch.to_string().to_lowercase());
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Check if token is a command
fn is_command(token: &str) -> bool {
    matches!(token, "m" | "n" | "l" | "b" | "s" | "p" | "c")
}

/// Parse coordinate string to f32
fn parse_coord(s: &str) -> Result<f32, RenderError> {
    s.parse::<f32>()
        .map_err(|_| RenderError::InvalidDrawCommand(format!("Invalid coordinate: {}", s)))
}

/// Convert B-spline control points to Bezier curves
///
/// Parameters:
/// - points: Control points for the spline
/// - extended: Whether to use extended spline algorithm (for 'p' command)
fn spline_to_bezier(
    points: &[(f32, f32)],
    extended: bool,
) -> Vec<((f32, f32), (f32, f32), (f32, f32))> {
    let mut beziers = Vec::new();

    if points.len() < 3 {
        return beziers;
    }

    if extended {
        // Extended B-spline for 'p' command - uses Catmull-Rom interpolation
        for i in 0..points.len() - 1 {
            let p0 = if i > 0 { points[i - 1] } else { points[i] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                points[i + 1]
            };

            // Catmull-Rom to Bezier conversion
            let tension = 0.5; // Standard Catmull-Rom tension
            let c1 = (
                p1.0 + tension * (p2.0 - p0.0) / 3.0,
                p1.1 + tension * (p2.1 - p0.1) / 3.0,
            );
            let c2 = (
                p2.0 - tension * (p3.0 - p1.0) / 3.0,
                p2.1 - tension * (p3.1 - p1.1) / 3.0,
            );

            beziers.push((c1, c2, p2));
        }
    } else {
        // Standard B-spline for 's' command
        // Uses uniform cubic B-spline to Bezier conversion
        for i in 0..points.len() - 1 {
            let p0 = if i > 0 { points[i - 1] } else { points[i] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                points[i + 1]
            };

            // Calculate control points for cubic bezier from B-spline
            let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
            let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);

            beziers.push((c1, c2, p2));
        }
    }

    beziers
}
