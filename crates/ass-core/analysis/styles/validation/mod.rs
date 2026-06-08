//! Style validation and conflict detection for ASS subtitle styles
//!
//! Provides comprehensive validation of style definitions including property
//! validation, inheritance analysis, and conflict detection. Designed for
//! editor integration and script quality assurance.
//!
//! # Features
//!
//! - Property validation with configurable severity levels
//! - Circular inheritance detection and resolution
//! - Duplicate name and missing reference detection
//! - Performance impact assessment for validation issues
//! - Zero-copy validation with lifetime-generic references
//!
//! # Performance
//!
//! - Target: <0.5ms per style validation
//! - Memory: Minimal allocations via zero-copy issue references
//! - Validation: Configurable depth limits for inheritance analysis

mod conflict;
mod inheritance;
mod issue;
mod severity;

#[cfg(test)]
mod tests;

pub use conflict::{ConflictType, StyleConflict};
pub use inheritance::StyleInheritance;
pub use issue::StyleValidationIssue;
pub use severity::ValidationSeverity;
