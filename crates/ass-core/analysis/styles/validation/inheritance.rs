//! Style inheritance tracking and analysis.
//!
//! Provides [`StyleInheritance`], a zero-copy tracker for the parent/child
//! relationships of a style and for detecting circular inheritance.

use alloc::vec::Vec;

/// Style inheritance tracking and analysis
#[derive(Debug, Clone)]
pub struct StyleInheritance<'a> {
    /// Style name (zero-copy reference)
    pub name: &'a str,
    /// Parent styles in inheritance chain
    pub parents: Vec<&'a str>,
    /// Child styles that inherit from this style
    pub children: Vec<&'a str>,
    /// Inheritance depth (0 = root style, no parents)
    pub depth: usize,
    /// Whether circular inheritance was detected
    pub has_circular_inheritance: bool,
}

impl<'a> StyleInheritance<'a> {
    /// Create new inheritance tracker for style
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            name,
            parents: Vec::new(),
            children: Vec::new(),
            depth: 0,
            has_circular_inheritance: false,
        }
    }

    /// Add parent style to inheritance chain
    pub fn add_parent(&mut self, parent: &'a str) {
        if !self.parents.contains(&parent) {
            self.parents.push(parent);
        }
    }

    /// Set parent style (for single inheritance)
    pub fn set_parent(&mut self, parent: &'a str) {
        self.parents.clear();
        self.parents.push(parent);
        self.depth = 1; // Will be adjusted later based on parent's depth
    }

    /// Add child style that inherits from this one
    pub fn add_child(&mut self, child: &'a str) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Check if style has inheritance relationships
    #[must_use]
    pub fn has_inheritance(&self) -> bool {
        !self.parents.is_empty() || !self.children.is_empty()
    }

    /// Check if style is root (no parents)
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Check if style is leaf (no children)
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}
