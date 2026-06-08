//! Lazy validation wrapper around ass-core's ScriptAnalysis
//!
//! Provides on-demand validation and linting for editor documents,
//! wrapping ass-core's analysis capabilities with caching and
//! incremental update support for better editor performance.

mod api;
mod config;
mod core_validation;
mod issue;
mod lazy_validator;
mod result;

#[cfg(test)]
mod tests;

pub use config::ValidatorConfig;
pub use issue::{ValidationIssue, ValidationSeverity};
pub use lazy_validator::LazyValidator;
pub use result::ValidationResult;
