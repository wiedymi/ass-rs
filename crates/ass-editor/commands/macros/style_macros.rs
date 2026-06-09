//! Style and script-info editing macros (`edit_style!`, `script_info!`).

/// Macro for editing styles
///
/// # Examples
///
/// ```
/// use ass_editor::{EditorDocument, edit_style, StyleBuilder};
///
/// let content = "[V4+ Styles]\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1";
/// let mut doc = EditorDocument::from_content(content).unwrap();
///
/// // Note: edit_style! macro has different syntax than shown
/// let style = StyleBuilder::new()
///     .name("Default")
///     .font("Arial")
///     .size(24)
///     .build()
///     .unwrap();
///
/// // The actual method to update styles would be different
/// // This is just an example of the intended usage
/// ```
#[macro_export]
macro_rules! edit_style {
    ($doc:expr, $name:expr, { $($field:ident = $value:expr),+ $(,)? }) => {{
        let style = $crate::StyleBuilder::new()
            .name($name)
            $(.$field($value))*
            .build()?;
        $doc.edit_style_line($name, &style)
    }};
}

/// Macro for script info field updates
///
/// # Examples
///
/// ```
/// use ass_editor::{EditorDocument, script_info, Position};
///
/// let mut doc = EditorDocument::from_content("[Script Info]\nTitle: \nAuthor: ").unwrap();
///
/// // Set script info fields - they must already exist in the document
/// doc.set_script_info_field("Title", "My Movie").unwrap();
/// doc.set_script_info_field("Author", "John Doe").unwrap();
///
/// assert!(doc.text().contains("Title: My Movie"));
/// assert!(doc.text().contains("Author: John Doe"));
/// ```
#[macro_export]
macro_rules! script_info {
    ($doc:expr, { $($key:expr => $value:expr),+ $(,)? }) => {{
        $(
            $doc.set_script_info_field($key, $value)?;
        )+
        Ok::<(), $crate::EditorError>(())
    }};
}
