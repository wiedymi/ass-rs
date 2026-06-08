//! Rendering-complexity scoring for ASS override tags.
//!
//! Maps tag names to a 1-5 cost score used by downstream rendering
//! optimization and analysis passes.

/// Calculate rendering complexity for a tag
///
/// Assigns complexity scores based on computational cost of rendering operations.
/// Higher scores indicate more expensive rendering operations.
///
/// # Complexity Scale
///
/// - 1: Basic formatting (bold, italic, colors)
/// - 2: Positioning and styling (position, alignment, borders)
/// - 3: Animations and transforms (movement, fades, rotations)
/// - 4: Advanced animations (transitions, complex effects)
/// - 5: Drawing commands (vector graphics)
///
/// # Arguments
///
/// * `tag_name` - Name of the ASS tag to score
///
/// # Returns
///
/// Complexity score from 1-5, defaults to 2 for unknown tags
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::tags::calculate_tag_complexity;
/// assert_eq!(calculate_tag_complexity("b"), 1);
/// assert_eq!(calculate_tag_complexity("pos"), 2);
/// assert_eq!(calculate_tag_complexity("move"), 3);
/// assert_eq!(calculate_tag_complexity("t"), 4);
/// assert_eq!(calculate_tag_complexity("p"), 5);
/// ```
#[must_use]
pub fn calculate_tag_complexity(tag_name: &str) -> u8 {
    match tag_name {
        "b" | "i" | "u" | "s" | "c" | "1c" | "2c" | "3c" | "4c" | "alpha" | "1a" | "2a" | "3a"
        | "4a" | "fn" | "fs" => 1,

        "move" | "fad" | "fade" | "frx" | "fry" | "frz" | "fscx" | "fscy" | "fsp" | "clip"
        | "iclip" => 3,
        "t" | "pbo" => 4,
        "p" => 5,
        _ => 2,
    }
}
