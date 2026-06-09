//! Comprehensive tests for plugin system functionality.
//!
//! Split into focused submodules, each kept under the 200-line limit:
//!
//! - `mocks`: shared mock tag handler and section processor.
//! - `results`: `TagResult`/`SectionResult` equality and `PluginError` display.
//! - `registry_basic`: registry construction and handler registration.
//! - `registry_process`: tag and section processing dispatch.
//! - `registry_management`: removal, clearing, naming, and debug output.
//! - `validation`: handler and processor argument validation.

mod mocks;
mod registry_basic;
mod registry_management;
mod registry_process;
mod results;
mod validation;
