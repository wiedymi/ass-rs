//! Data types for the auto-completion extension.
//!
//! Defines the completion suggestion model ([`CompletionItem`],
//! [`CompletionType`]), the cursor [`CompletionContext`], and the
//! [`AutoCompleteConfig`] tuning options.

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Type of completion being provided
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionType {
    /// Section header completion
    Section,
    /// Field name completion
    Field,
    /// Field value completion
    Value,
    /// Style name reference
    StyleRef,
    /// Override tag
    Tag,
    /// Tag parameter
    TagParam,
    /// Color value
    Color,
    /// Time code
    Time,
}

/// A single completion suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// Text to insert
    pub insert_text: String,
    /// Display label
    pub label: String,
    /// Type of completion
    pub completion_type: CompletionType,
    /// Description of the item
    pub description: Option<String>,
    /// Additional details
    pub detail: Option<String>,
    /// Sort priority (lower = higher priority)
    pub sort_order: u32,
}

impl CompletionItem {
    /// Create a new completion item
    pub fn new(insert_text: String, label: String, completion_type: CompletionType) -> Self {
        Self {
            insert_text,
            label,
            completion_type,
            description: None,
            detail: None,
            sort_order: 999,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set detail
    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Set sort order
    pub fn with_sort_order(mut self, order: u32) -> Self {
        self.sort_order = order;
        self
    }
}

/// Completion context at a position
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current line text
    pub line: String,
    /// Position within the line
    pub column: usize,
    /// Current section (if any)
    pub section: Option<String>,
    /// Whether we're inside an override tag
    pub in_override_tag: bool,
    /// Current tag being typed (if any)
    pub current_tag: Option<String>,
}

/// Configuration for auto-completion
#[derive(Debug, Clone)]
pub struct AutoCompleteConfig {
    /// Enable field name completion
    pub complete_fields: bool,
    /// Enable style reference completion
    pub complete_styles: bool,
    /// Enable override tag completion
    pub complete_tags: bool,
    /// Enable value completion
    pub complete_values: bool,
    /// Maximum suggestions to show
    pub max_suggestions: usize,
    /// Minimum characters before triggering
    pub min_chars: usize,
}

impl Default for AutoCompleteConfig {
    fn default() -> Self {
        Self {
            complete_fields: true,
            complete_styles: true,
            complete_tags: true,
            complete_values: true,
            max_suggestions: 20,
            min_chars: 1,
        }
    }
}
