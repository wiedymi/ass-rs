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

/// Register built-in extensions with the RegistryIntegration
///
/// This function registers the built-in extensions (syntax highlighting and auto-completion)
/// with the registry integration system, making them available during ASS parsing.
pub fn register_builtin_extensions(
    registry: &mut crate::extensions::registry_integration::RegistryIntegration,
) -> crate::core::Result<()> {
    // Register all built-in tag and section handlers from ass-core
    registry.register_builtin_handlers()?;
    registry.register_builtin_sections()?;

    // Note: The syntax highlighting and auto-completion extensions don't provide
    // tag handlers or section processors directly. They work at the editor level
    // to provide IDE-like features rather than parsing extensions.
    //
    // If we wanted to add custom tag/section handlers from editor extensions,
    // we would do it here using:
    // - registry.register_custom_tag_handler(...)
    // - registry.register_custom_section_processor(...)

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use crate::extensions::ExtensionManager;
    use crate::extensions::{EditorExtension, ExtensionCapability};
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    #[test]
    fn test_load_builtin_extensions() {
        let manager = ExtensionManager::new();

        // Load built-in extensions
        // load_builtin_extensions(&mut manager).unwrap(); // TODO: Implement for nostd

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

    #[test]
    fn test_register_builtin_extensions() {
        use crate::extensions::registry_integration::RegistryIntegration;

        let mut registry = RegistryIntegration::new();

        // Should successfully register all built-in extensions
        assert!(register_builtin_extensions(&mut registry).is_ok());

        // Test that the registry can be used to parse ASS content with tags
        let test_content = r#"[Script Info]
Title: Test Script
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour
Style: Default,Arial,20,&H00FFFFFF,&H000000FF

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\b1}Bold{\b0} text with {\i1}italics{\i0}"#;

        let result = ass_core::parser::Script::builder()
            .with_registry(registry.registry())
            .parse(test_content);

        assert!(result.is_ok());
        let script = result.unwrap();

        // Verify script was parsed correctly
        assert_eq!(script.sections().len(), 3);
    }
}
