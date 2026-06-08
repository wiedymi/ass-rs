//! Extension capability and metadata types.
//!
//! Defines the capabilities an extension can advertise and the descriptive
//! metadata (`ExtensionInfo`) attached to each extension.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Extension capabilities that can be provided
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionCapability {
    /// Text processing and transformation
    TextProcessing,
    /// Syntax highlighting and theming
    SyntaxHighlighting,
    /// Code completion and suggestions
    CodeCompletion,
    /// Linting and validation
    Linting,
    /// Import/export format support
    FormatSupport,
    /// Custom commands and shortcuts
    CustomCommands,
    /// UI enhancements and widgets
    UserInterface,
    /// External tool integration
    ToolIntegration,
    /// Custom event handling
    EventHandling,
    /// Performance monitoring
    Performance,
}

impl ExtensionCapability {
    /// Get a human-readable description of the capability
    pub fn description(&self) -> &'static str {
        match self {
            Self::TextProcessing => "Text processing and transformation",
            Self::SyntaxHighlighting => "Syntax highlighting and theming",
            Self::CodeCompletion => "Code completion and suggestions",
            Self::Linting => "Linting and validation",
            Self::FormatSupport => "Import/export format support",
            Self::CustomCommands => "Custom commands and shortcuts",
            Self::UserInterface => "UI enhancements and widgets",
            Self::ToolIntegration => "External tool integration",
            Self::EventHandling => "Custom event handling",
            Self::Performance => "Performance monitoring",
        }
    }
}

/// Extension metadata and information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionInfo {
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Extension author
    pub author: String,
    /// Extension description
    pub description: String,
    /// Capabilities provided by this extension
    pub capabilities: Vec<ExtensionCapability>,
    /// Dependencies on other extensions
    pub dependencies: Vec<String>,
    /// Optional extension website/homepage
    pub homepage: Option<String>,
    /// License identifier
    pub license: Option<String>,
}

impl ExtensionInfo {
    /// Create a new extension info
    pub fn new(name: String, version: String, author: String, description: String) -> Self {
        Self {
            name,
            version,
            author,
            description,
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            homepage: None,
            license: None,
        }
    }

    /// Add a capability to this extension
    pub fn with_capability(mut self, capability: ExtensionCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<ExtensionCapability>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    /// Add a dependency on another extension
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Set the homepage URL
    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = Some(homepage);
        self
    }

    /// Set the license
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// Check if this extension provides a specific capability
    pub fn has_capability(&self, capability: &ExtensionCapability) -> bool {
        self.capabilities.contains(capability)
    }
}
