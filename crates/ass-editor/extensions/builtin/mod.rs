//! Built-in extensions for the ASS editor
//!
//! This module provides commonly-used extensions that ship with the editor:
//! - Syntax highlighting for ASS/SSA files
//! - Auto-completion for ASS format elements

pub mod auto_complete;
pub mod syntax_highlight;

pub use auto_complete::AutoCompleteExtension;
pub use syntax_highlight::SyntaxHighlightExtension;

/// Load all built-in extensions into an extension manager
#[cfg(feature = "std")]
pub fn load_builtin_extensions(
    manager: &mut crate::extensions::ExtensionManager,
) -> crate::core::Result<()> {
    // Load syntax highlighting
    let syntax_ext = Box::new(SyntaxHighlightExtension::new());
    manager.load_extension(syntax_ext)?;

    // Load auto-completion
    let autocomplete_ext = Box::new(AutoCompleteExtension::new());
    manager.load_extension(autocomplete_ext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::{EditorExtension, ExtensionCapability, ExtensionManager};

    #[test]
    fn test_load_builtin_extensions() {
        let mut manager = ExtensionManager::new();

        // Load built-in extensions
        load_builtin_extensions(&mut manager).unwrap();

        // Check that both extensions are loaded
        let extensions = manager.list_extensions();
        assert_eq!(extensions.len(), 2);
        assert!(extensions.contains(&"syntax-highlight".to_string()));
        assert!(extensions.contains(&"auto-complete".to_string()));
    }

    #[test]
    fn test_syntax_highlight_extension() {
        let ext = SyntaxHighlightExtension::new();
        assert_eq!(ext.info().name, "syntax-highlight");
        assert!(ext
            .info()
            .has_capability(&ExtensionCapability::SyntaxHighlighting));
    }

    #[test]
    fn test_auto_complete_extension() {
        let ext = AutoCompleteExtension::new();
        assert_eq!(ext.info().name, "auto-complete");
        assert!(ext
            .info()
            .has_capability(&ExtensionCapability::CodeCompletion));
    }
}
