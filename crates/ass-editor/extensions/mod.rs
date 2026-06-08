//! Extension system for editor functionality
//!
//! Provides the `EditorExtension` trait for extending editor capabilities
//! with custom functionality. Supports both synchronous and asynchronous
//! operations, lifecycle management, and inter-extension communication.

pub mod builtin;
pub mod registry_integration;

#[cfg(not(feature = "std"))]
extern crate alloc;

mod command;
mod context;
mod extension;
mod info;
mod manager;
mod manager_access;
mod manager_lifecycle;
mod manager_loading;

#[cfg(test)]
mod extensions_tests;
#[cfg(test)]
mod manager_tests;

pub use command::{ExtensionCommand, ExtensionResult, ExtensionState, MessageLevel};
pub use context::{EditorContext, ExtensionContext};
pub use extension::{EditorExtension, MessageHandler};
pub use info::{ExtensionCapability, ExtensionInfo};
pub use manager::{EventSender, ExtensionManager};

#[cfg(feature = "std")]
pub use extension::StdMessageHandler;

#[cfg(not(feature = "std"))]
pub use extension::NoOpMessageHandler;
