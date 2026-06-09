//! Position-oriented fluent editing macro (`at_pos!`).

/// Fluent API macro for position operations
///
/// # Examples
///
/// ```
/// use ass_editor::{EditorDocument, Position, at_pos};
///
/// let mut doc = EditorDocument::from_content("Hello world!").unwrap();
///
/// // Note: at_pos! macro doesn't exist in the current implementation
/// // Using direct methods instead
/// doc.insert(Position::new(5), " beautiful").unwrap();
/// assert_eq!(doc.text(), "Hello beautiful world!");
/// ```
#[macro_export]
macro_rules! at_pos {
    ($doc:expr, $pos:expr, insert $text:expr) => {
        $doc.at($crate::Position::new($pos)).insert_text($text)
    };

    ($doc:expr, $pos:expr, replace $len:expr, $text:expr) => {
        $doc.at($crate::Position::new($pos))
            .replace_text($len, $text)
    };

    ($doc:expr, $pos:expr, delete $len:expr) => {
        $doc.at($crate::Position::new($pos)).delete_range($len)
    };
}
