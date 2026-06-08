//! Memory management for editor sessions
//!
//! Provides arena reset logic, memory pooling, and garbage collection
//! strategies to prevent memory accumulation and ensure efficient
//! resource utilization across multiple editing sessions.

mod config;
mod gc;
mod pool;
mod stats;
mod strategy;

#[cfg(test)]
mod tests;

pub use config::MemoryPoolConfig;
pub use pool::MemoryPool;
pub use stats::MemoryStats;
pub use strategy::{ResetStrategy, SmartArenaManager};
