//! Session management for multi-document editing
//!
//! Provides the `EditorSessionManager` for managing multiple documents
//! with shared resources, arenas, and extension registries. Supports
//! efficient session switching (<100µs target) and resource pooling.

#[cfg(not(feature = "std"))]
extern crate alloc;

mod access;
mod config;
mod lifecycle;
mod manager;
mod session;

#[cfg(test)]
mod manager_tests;

pub use config::{SessionConfig, SessionStats};
pub use manager::EditorSessionManager;
pub use session::EditorSession;
