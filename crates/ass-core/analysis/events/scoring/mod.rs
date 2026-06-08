//! Complexity scoring algorithms for ASS events
//!
//! Provides efficient calculation of animation complexity and rendering performance
//! impact scores for dialogue events. Scores are computed based on override tag
//! analysis, text content, and performance heuristics.
//!
//! # Scoring System
//!
//! - Animation Score: 0-10 scale based on tag complexity
//! - Complexity Score: 0-100 scale combining multiple factors
//! - Performance Impact: Categorical assessment for rendering optimization
//!
//! # Performance
//!
//! - Target: <0.1ms per scoring operation
//! - Memory: Zero allocations, operates on borrowed data
//! - Scalability: Linear complexity O(n) where n = tag count

mod calculators;
mod impact;

#[cfg(test)]
mod tests;

pub use calculators::{calculate_animation_score, calculate_complexity_score};
pub use impact::{get_performance_impact, PerformanceImpact};
