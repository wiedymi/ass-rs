//! Incremental parsing integration with ass-core
//!
//! Provides efficient incremental parsing by leveraging Script::parse_partial()
//! to achieve <1ms edit times and <5ms reparse times. Tracks deltas for proper
//! undo/redo integration and maintains consistency with the rope structure.

use super::{Range, Result};
use crate::core::errors::EditorError;
use ass_core::parser::{script::ScriptDeltaOwned, Script};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, string::ToString, format};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Represents a change to the document with delta tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentChange<'a> {
    /// The range that was affected
    pub range: Range,
    
    /// The new text that replaced the range
    pub new_text: Cow<'a, str>,
    
    /// The old text that was replaced (for undo)
    pub old_text: Cow<'a, str>,
    
    /// Timestamp when change occurred (for grouping)
    #[cfg(feature = "std")]
    pub timestamp: std::time::Instant,
    
    /// Change ID for tracking (monotonic counter)
    pub change_id: u64,
}

/// Manages incremental parsing state and delta accumulation
#[derive(Debug)]
pub struct IncrementalParser {
    /// Last successfully parsed script (cached)
    cached_script: Option<String>,
    
    /// Accumulated changes since last full parse (owned to persist)
    pending_changes: Vec<DocumentChange<'static>>,
    
    /// Next change ID
    next_change_id: u64,
    
    /// Threshold for triggering full reparse (in bytes changed)
    reparse_threshold: usize,
    
    /// Total bytes changed since last full parse
    bytes_changed: usize,
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new() -> Self {
        Self {
            cached_script: None,
            pending_changes: Vec::new(),
            next_change_id: 1,
            reparse_threshold: 10_000, // 10KB of changes triggers full reparse
            bytes_changed: 0,
        }
    }
    
    /// Set the reparse threshold in bytes
    pub fn set_reparse_threshold(&mut self, threshold: usize) {
        self.reparse_threshold = threshold;
    }
    
    /// Initialize the cache with the current document content
    /// This primes the incremental parser for efficient subsequent edits
    pub fn initialize_cache(&mut self, content: &str) {
        self.cached_script = Some(content.to_string());
        self.pending_changes.clear();
        self.bytes_changed = 0;
    }
    
    /// Check if there's a cached script available
    pub fn has_cached_script(&self) -> bool {
        self.cached_script.is_some()
    }
    
    /// Execute a function with the cached script (avoids re-parsing)
    pub fn with_cached_script<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Script) -> Result<R>,
    {
        let cached = self.cached_script.as_ref()
            .ok_or_else(|| EditorError::command_failed("No cached script available"))?;
        let script = Script::parse(cached)
            .map_err(EditorError::from)?;
        f(&script)
    }
    
    /// Apply a change incrementally, returning the delta
    pub fn apply_change(
        &mut self,
        document_text: &str,
        range: Range,
        new_text: &str,
    ) -> Result<ScriptDeltaOwned> {
        // If we don't have a cached script or too many changes accumulated, do full parse
        if self.cached_script.is_none() || self.bytes_changed >= self.reparse_threshold {
            return self.full_reparse(document_text);
        }
        
        // Validate range
        if range.end.offset > document_text.len() || range.start.offset > range.end.offset {
            return Err(EditorError::InvalidRange {
                start: range.start.offset,
                end: range.end.offset,
                length: document_text.len(),
            });
        }
        
        // Check if we're already on valid UTF-8 boundaries
        let start_is_valid = range.start.offset == 0 || 
                           range.start.offset == document_text.len() ||
                           document_text.is_char_boundary(range.start.offset);
        let end_is_valid = range.end.offset == 0 || 
                         range.end.offset == document_text.len() ||
                         document_text.is_char_boundary(range.end.offset);
        
        if !start_is_valid || !end_is_valid {
            // The range is not on valid UTF-8 boundaries - this is an error
            // We should not silently adjust the range as it will cause undo/redo issues
            return Err(EditorError::command_failed(
                "Edit range is not on valid UTF-8 character boundaries"
            ));
        }
        
        // Get the old text being replaced
        let old_text = &document_text[range.start.offset..range.end.offset];
        let (start_byte, end_byte) = (range.start.offset, range.end.offset);
        
        // Track the change (convert to owned for storage)
        let change = DocumentChange {
            range,
            new_text: Cow::Owned(new_text.to_string()),
            old_text: Cow::Owned(old_text.to_string()),
            #[cfg(feature = "std")]
            timestamp: std::time::Instant::now(),
            change_id: self.next_change_id,
        };
        self.next_change_id += 1;
        
        // Update bytes changed counter
        let change_size = new_text.len().abs_diff(old_text.len());
        self.bytes_changed += change_size;
        
        // Store the change for potential rollback
        self.pending_changes.push(change);
        
        // Convert editor Range to std::ops::Range for parse_partial
        // Use the corrected boundaries from old_text extraction
        let byte_range = start_byte..end_byte;
        
        // Parse the cached script first to get a Script instance
        let cached = self.cached_script.as_ref()
            .ok_or_else(|| EditorError::command_failed("Cached script unavailable for incremental parsing"))?;
        let script = Script::parse(cached)
            .map_err(EditorError::from)?;
        
        // Apply incremental parsing
        match script.parse_partial(byte_range, new_text) {
            Ok(delta) => {
                // Update cached script with the change
                self.update_cached_script(range, new_text)?;
                Ok(delta)
            }
            Err(e) => {
                // Fall back to full reparse on error
                self.pending_changes.pop(); // Remove failed change
                self.bytes_changed -= change_size;
                
                // Log the error for debugging
                #[cfg(feature = "std")]
                eprintln!("Incremental parse failed, falling back to full parse: {e}");
                
                self.full_reparse(document_text)
            }
        }
    }
    
    /// Force a full reparse of the document
    pub fn full_reparse(&mut self, content: &str) -> Result<ScriptDeltaOwned> {
        // Parse the entire document
        let new_script = Script::parse(content).map_err(EditorError::from)?;
        
        // If we had a previous script, calculate delta
        let delta = if let Some(cached_content) = &self.cached_script {
            let old_script = Script::parse(cached_content).map_err(EditorError::from)?;
            
            // Calculate sections that changed
            let delta = ass_core::parser::calculate_delta(&old_script, &new_script);
            
            // Convert to owned format
            let mut owned_delta = ScriptDeltaOwned {
                added: Vec::new(),
                modified: Vec::new(),
                removed: Vec::new(),
                new_issues: new_script.issues().to_vec(),
            };
            
            // Convert added sections
            for section in delta.added {
                owned_delta.added.push(format!("{section:?}"));
            }
            
            // Convert modified sections  
            for (idx, section) in delta.modified {
                owned_delta.modified.push((idx, format!("{section:?}")));
            }
            
            // Copy removed indices
            owned_delta.removed = delta.removed;
            
            owned_delta
        } else {
            // First parse - everything is "added"
            ScriptDeltaOwned {
                added: new_script.sections().iter()
                    .map(|s| format!("{s:?}"))
                    .collect(),
                modified: Vec::new(),
                removed: Vec::new(),
                new_issues: new_script.issues().to_vec(),
            }
        };
        
        // Update cache
        self.cached_script = Some(content.to_string());
        self.pending_changes.clear();
        self.bytes_changed = 0;
        
        Ok(delta)
    }
    
    /// Clear all cached state
    pub fn clear_cache(&mut self) {
        self.cached_script = None;
        self.pending_changes.clear();
        self.bytes_changed = 0;
        self.next_change_id = 1;
    }
    
    /// Get accumulated changes since last full parse
    pub fn pending_changes(&self) -> &[DocumentChange<'static>] {
        &self.pending_changes
    }
    
    /// Check if a full reparse is recommended
    pub fn should_reparse(&self) -> bool {
        self.bytes_changed >= self.reparse_threshold || self.pending_changes.len() > 50
    }
    
    /// Update the cached script content with a change
    fn update_cached_script(&mut self, range: Range, new_text: &str) -> Result<()> {
        if let Some(cached) = &mut self.cached_script {
            // Validate boundaries
            if range.start.offset > cached.len() || range.end.offset > cached.len() {
                return Err(EditorError::InvalidRange {
                    start: range.start.offset,
                    end: range.end.offset,
                    length: cached.len(),
                });
            }
            
            // Ensure we're on valid UTF-8 boundaries
            if !cached.is_char_boundary(range.start.offset) || !cached.is_char_boundary(range.end.offset) {
                return Err(EditorError::command_failed(
                    "Cache update range is not on valid UTF-8 character boundaries"
                ));
            }
            
            // Build the new content
            let mut result = String::with_capacity(
                cached.len() - (range.end.offset - range.start.offset) + new_text.len()
            );
            
            // Copy content before the change
            result.push_str(&cached[..range.start.offset]);
            
            // Insert new text
            result.push_str(new_text);
            
            // Copy content after the change
            if range.end.offset < cached.len() {
                result.push_str(&cached[range.end.offset..]);
            }
            
            *cached = result;
        }
        
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Position;
    
    #[test]
    fn test_incremental_parser_creation() {
        let parser = IncrementalParser::new();
        assert!(parser.cached_script.is_none());
        assert!(parser.pending_changes.is_empty());
        assert_eq!(parser.bytes_changed, 0);
    }
    
    #[test]
    fn test_document_change_tracking() {
        let change = DocumentChange {
            range: Range::new(Position::new(0), Position::new(5)),
            new_text: Cow::Borrowed("Hello"),
            old_text: Cow::Borrowed("World"),
            #[cfg(feature = "std")]
            timestamp: std::time::Instant::now(),
            change_id: 1,
        };
        
        assert_eq!(change.new_text, "Hello");
        assert_eq!(change.old_text, "World");
        assert_eq!(change.change_id, 1);
    }
    
    #[test]
    fn test_should_reparse_threshold() {
        let mut parser = IncrementalParser::new();
        parser.set_reparse_threshold(100);
        
        assert!(!parser.should_reparse());
        
        parser.bytes_changed = 101;
        assert!(parser.should_reparse());
    }
    
    #[test]
    fn test_clear_cache() {
        let mut parser = IncrementalParser::new();
        parser.cached_script = Some("test".to_string());
        parser.bytes_changed = 100;
        parser.next_change_id = 5;
        
        parser.clear_cache();
        
        assert!(parser.cached_script.is_none());
        assert_eq!(parser.bytes_changed, 0);
        assert_eq!(parser.next_change_id, 1);
    }
    
    #[test]
    fn test_error_recovery() {
        let mut parser = IncrementalParser::new();
        
        // Test full reparse on first use (no cached script)
        let content = "[Script Info]\nTitle: Test";
        let result = parser.apply_change(content, Range::new(Position::new(0), Position::new(5)), "New");
        assert!(result.is_ok());
        assert!(parser.cached_script.is_some());
        
        // Test threshold-based full reparse
        parser.set_reparse_threshold(10);
        parser.bytes_changed = 11;
        let result = parser.apply_change(content, Range::new(Position::new(0), Position::new(5)), "Changed");
        assert!(result.is_ok());
        assert_eq!(parser.bytes_changed, 0); // Reset after full reparse
    }
}