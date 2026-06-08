//! Thread safety abstractions for the editor
//!
//! Provides thread-safe wrappers and synchronization primitives for
//! multi-threaded editor usage, ensuring safe concurrent access to
//! document state and operations.

// Allow Arc with non-Send/Sync types because we're providing
// thread safety through RwLock synchronization
#![allow(clippy::arc_with_non_send_sync)]

#[cfg(all(feature = "concurrency", feature = "async"))]
mod async_document;
#[cfg(feature = "concurrency")]
mod document_pool;
#[cfg(feature = "concurrency")]
mod scoped_lock;
#[cfg(feature = "concurrency")]
mod sync_document;

#[cfg(test)]
#[cfg(feature = "concurrency")]
mod tests;

#[cfg(all(feature = "concurrency", feature = "async"))]
pub use async_document::AsyncDocument;
#[cfg(feature = "concurrency")]
pub use document_pool::DocumentPool;
#[cfg(feature = "concurrency")]
pub use scoped_lock::ScopedDocumentLock;
#[cfg(feature = "concurrency")]
pub use sync_document::SyncDocument;
