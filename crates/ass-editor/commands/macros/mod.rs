//! Proc-macros for ergonomic ASS editing
//!
//! Provides the `edit_event!` macro and other shortcuts for common ASS operations
//! as specified in the architecture (line 127).
//!
//! All macros are `#[macro_export]`, so they are available at the crate root
//! (e.g. `ass_editor::edit_event!`) regardless of which submodule defines them.

mod event_macros;
mod position_macros;
mod style_macros;

#[cfg(test)]
mod tests;
