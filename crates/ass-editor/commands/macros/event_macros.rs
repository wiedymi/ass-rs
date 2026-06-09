//! Event-oriented editing macros (`edit_event!`, `add_event!`).

/// Macro for editing events with multiple field updates
///
/// Supports both simple text editing and complex field updates:
/// - `edit_event!(doc, index, "new text")` - Simple text replacement
/// - `edit_event!(doc, index, text = "new", start = "0:00:05.00", end = "0:00:10.00")` - Multi-field
///
/// # Examples
///
/// ```
/// use ass_editor::{EditorDocument, edit_event};
///
/// let content = "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Old text";
/// let mut doc = EditorDocument::from_content(content).unwrap();
///
/// // Edit event using the macro - this updates the text field
/// edit_event!(doc, 0, |event| {
///     vec![("text", "New dialogue text".to_string())]
/// }).unwrap();
/// assert!(doc.text().contains("New dialogue text"));
/// ```
#[macro_export]
macro_rules! edit_event {
    // Simple text replacement: edit_event!(doc, index, "text")
    ($doc:expr, $index:expr, $text:expr) => {
        $doc.edit_event_by_index($index, $text)
    };

    // Multi-field edit: edit_event!(doc, index, field = value, ...)
    ($doc:expr, $index:expr, $($field:ident = $value:expr),+ $(,)?) => {{
        let mut builder = $crate::core::EventUpdateBuilder::new();
        $(
            builder = edit_event!(@field builder, $field, $value);
        )+
        $doc.edit_event_with_builder($index, builder)
    }};

    // Internal helper for field assignments
    (@field $builder:expr, text, $value:expr) => {
        $builder.text($value)
    };
    (@field $builder:expr, start, $value:expr) => {
        $builder.start_time($value)
    };
    (@field $builder:expr, end, $value:expr) => {
        $builder.end_time($value)
    };
    (@field $builder:expr, speaker, $value:expr) => {
        $builder.speaker($value)
    };
    (@field $builder:expr, style, $value:expr) => {
        $builder.style($value)
    };
    (@field $builder:expr, layer, $value:expr) => {
        $builder.layer($value)
    };
    (@field $builder:expr, effect, $value:expr) => {
        $builder.effect($value)
    };
}

/// Macro for quickly adding events
///
/// # Examples
///
/// ```
/// use ass_editor::{EditorDocument, add_event};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut doc = EditorDocument::from_content("[Events]")?;
///
/// add_event!(doc, dialogue {
///     start_time = "0:00:05.00",
///     end_time = "0:00:10.00",
///     text = "Hello world!"
/// })?;
///
/// assert!(doc.text().contains("Hello world!"));
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! add_event {
    ($doc:expr, dialogue { $($field:ident = $value:expr),+ $(,)? }) => {{
        let event = $crate::EventBuilder::dialogue()
            $(.$field($value))*
            .build()?;
        $doc.add_event_line(&event)
    }};

    ($doc:expr, comment { $($field:ident = $value:expr),+ $(,)? }) => {{
        let event = $crate::EventBuilder::comment()
            $(.$field($value))*
            .build()?;
        $doc.add_event_line(&event)
    }};
}
