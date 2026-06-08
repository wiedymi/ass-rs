//! Session configuration and statistics types.
//!
//! Defines [`SessionConfig`] for tuning session-manager behaviour and
//! [`SessionStats`] for reporting active sessions, memory usage, and
//! arena-reset activity.

/// Configuration for session management
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,

    /// Maximum memory usage per session in bytes
    pub max_memory_per_session: usize,

    /// Total memory limit across all sessions
    pub total_memory_limit: usize,

    /// Whether to enable automatic cleanup of unused sessions
    pub auto_cleanup: bool,

    /// Interval for arena resets (0 = never reset)
    pub arena_reset_interval: usize,

    /// Whether to share extension registry across sessions
    pub share_extensions: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 50,
            max_memory_per_session: 100 * 1024 * 1024, // 100MB per session
            total_memory_limit: 1024 * 1024 * 1024,    // 1GB total
            auto_cleanup: true,
            arena_reset_interval: 1000, // Reset every 1000 operations
            share_extensions: true,
        }
    }
}

/// Statistics about session manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStats {
    /// Number of active sessions
    pub active_sessions: usize,

    /// Total memory usage across all sessions
    pub total_memory_usage: usize,

    /// Number of operations since last cleanup
    pub operations_since_cleanup: usize,

    /// Number of arena resets performed
    pub arena_resets: usize,
}
