//! Plugin trait definitions and registry.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionKind {
    ScriptInfo,
    V4Styles,
    Events,
    Custom(&'static str),
}

/// Animation interpolation modes for tag animations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationMode {
    Linear,
    Accelerating(f32),
    Decelerating(f32),
    Bezier(f32, f32, f32, f32),
}

/// Tag argument types for better type safety and validation
#[derive(Debug, Clone, PartialEq)]
pub enum TagArgument {
    Integer(i32),
    Float(f32),
    Color([u8; 3]),
    Boolean(bool),
    String(String),
    Position(f32, f32),
    Animation {
        start_time: f32,
        end_time: f32,
        start_value: Box<TagArgument>,
        end_value: Box<TagArgument>,
        mode: AnimationMode,
    },
}

/// Enhanced animation state for complex tag animations
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub current_time: f64,
    pub interpolated_values: HashMap<String, TagArgument>,
    pub active_animations: Vec<ActiveAnimation>,
}

#[derive(Debug, Clone)]
pub struct ActiveAnimation {
    pub tag_name: String,
    pub start_time: f64,
    pub end_time: f64,
    pub start_value: TagArgument,
    pub end_value: TagArgument,
    pub mode: AnimationMode,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            interpolated_values: HashMap::new(),
            active_animations: Vec::new(),
        }
    }

    /// Update animation state for given time
    pub fn update(&mut self, time: f64) {
        self.current_time = time;
        self.interpolated_values.clear();

        for anim in &self.active_animations {
            if time >= anim.start_time && time <= anim.end_time {
                let progress = (time - anim.start_time) / (anim.end_time - anim.start_time);
                let eased_progress = apply_easing(progress as f32, anim.mode);

                if let Some(interpolated) =
                    interpolate_value(&anim.start_value, &anim.end_value, eased_progress)
                {
                    self.interpolated_values
                        .insert(anim.tag_name.clone(), interpolated);
                }
            }
        }
    }

    /// Add a new animation to the state
    pub fn add_animation(&mut self, animation: ActiveAnimation) {
        self.active_animations.push(animation);
    }
}

/// Apply easing function based on animation mode
fn apply_easing(t: f32, mode: AnimationMode) -> f32 {
    match mode {
        AnimationMode::Linear => t,
        AnimationMode::Accelerating(factor) => t.powf(factor),
        AnimationMode::Decelerating(factor) => 1.0 - (1.0 - t).powf(factor),
        AnimationMode::Bezier(_p1x, p1y, _p2x, p2y) => {
            // Simplified cubic bezier approximation
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            mt3 * 0.0 + 3.0 * mt2 * t * p1y + 3.0 * mt * t2 * p2y + t3 * 1.0
        }
    }
}

/// Interpolate between two tag argument values
fn interpolate_value(start: &TagArgument, end: &TagArgument, progress: f32) -> Option<TagArgument> {
    match (start, end) {
        (TagArgument::Float(s), TagArgument::Float(e)) => {
            Some(TagArgument::Float(s + (e - s) * progress))
        }
        (TagArgument::Integer(s), TagArgument::Integer(e)) => {
            Some(TagArgument::Integer(s + ((e - s) as f32 * progress) as i32))
        }
        (TagArgument::Position(sx, sy), TagArgument::Position(ex, ey)) => Some(
            TagArgument::Position(sx + (ex - sx) * progress, sy + (ey - sy) * progress),
        ),
        (TagArgument::Color([sr, sg, sb]), TagArgument::Color([er, eg, eb])) => {
            Some(TagArgument::Color([
                (sr + ((*er as f32 - *sr as f32) * progress) as u8),
                (sg + ((*eg as f32 - *sg as f32) * progress) as u8),
                (sb + ((*eb as f32 - *sb as f32) * progress) as u8),
            ]))
        }
        _ => None,
    }
}

/// Trait implemented by per-section parser/serializer plugins.
///
/// For now, the trait surface is minimal; it will grow (parse/serialize hooks) later.
pub trait Section: Send + Sync {
    /// Unique kind discriminator.
    fn kind(&self) -> SectionKind;

    /// Feed one line/token to the section parser. The line belongs to this section.
    /// Implementations may ignore unknown tokens.
    fn parse_token(&mut self, token: &crate::tokenizer::AssToken, src: &[u8]);

    /// Serialize the section back to ASS text form, writing into `dst`.
    fn serialize(&self, dst: &mut String);
}

/// Factory capable of producing fresh Section instances.
///
/// Using a factory allows the core parser to instantiate a new object per ASS file while keeping
/// plugin registration static.
pub trait SectionFactory: Send + Sync {
    fn create(&self) -> Box<dyn Section>;
    fn kind(&self) -> SectionKind {
        self.create().kind()
    }
}

/// Enhanced trait for override tag handlers with animation and timing support.
pub trait Tag: Send + Sync {
    /// Name of the tag (without leading backslash), e.g. "b" or "i".
    fn name(&self) -> &'static str;

    /// Parse and validate tag arguments, returning structured data
    fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
        // Default implementation for backward compatibility
        if self.parse_args_legacy(args) {
            Ok(vec![])
        } else {
            Err(TagParseError::InvalidArguments)
        }
    }

    /// Legacy parsing method for backward compatibility
    fn parse_args_legacy(&self, _args: &[u8]) -> bool {
        true
    }

    /// Apply tag effect with current animation state
    fn apply(
        &self,
        _args: &[TagArgument],
        _state: &mut AnimationState,
    ) -> Result<(), TagApplicationError> {
        // Default implementation - no animation support
        Ok(())
    }

    /// Check if this tag supports animation
    fn supports_animation(&self) -> bool {
        false
    }

    /// Get the expected argument types for validation
    fn expected_args(&self) -> &[TagArgumentType] {
        &[]
    }
}

/// Argument type specification for tag validation
#[derive(Debug, Clone, PartialEq)]
pub enum TagArgumentType {
    Integer,
    Float,
    Color,
    Boolean,
    String,
    Position,
    Optional(Box<TagArgumentType>),
}

/// Errors that can occur during tag parsing
#[derive(Debug, Clone, PartialEq)]
pub enum TagParseError {
    InvalidArguments,
    WrongArgumentCount,
    TypeMismatch(TagArgumentType),
    InvalidAnimationSyntax,
}

/// Errors that can occur during tag application
#[derive(Debug, Clone, PartialEq)]
pub enum TagApplicationError {
    AnimationConflict,
    InvalidState,
    UnsupportedOperation,
}

// -------------------------------------------------------------------------------------------------
// Registry implementation (std only for now)
// -------------------------------------------------------------------------------------------------

#[cfg(feature = "std")]
mod registry {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    #[derive(Default)]
    pub struct Registry {
        /// Section factories keyed by SectionKind.
        section_factories: Vec<&'static dyn SectionFactory>,
        /// Tag plugins keyed by tag name (lowercase ascii).
        tags: HashMap<&'static str, &'static dyn Tag>,
        /// Dynamic plugin handles for cleanup
        #[cfg(feature = "dynamic-loading")]
        dynamic_plugins: Vec<libloading::Library>,
    }

    impl Registry {
        fn new() -> Self {
            Self::default()
        }

        /// Load plugin from dynamic library (desktop only)
        #[cfg(all(feature = "dynamic-loading", not(target_family = "wasm")))]
        pub fn load_plugin_library(
            &mut self,
            path: &std::path::Path,
        ) -> Result<(), DynamicLoadError> {
            use libloading::{Library, Symbol};

            let lib = unsafe { Library::new(path) }.map_err(DynamicLoadError::LoadFailed)?;

            // Look for plugin registration function
            let register_fn: Symbol<unsafe extern "C" fn()> = unsafe {
                lib.get(b"register_plugin")
                    .map_err(DynamicLoadError::SymbolNotFound)?
            };

            // Call the registration function
            unsafe {
                register_fn();
            }

            self.dynamic_plugins.push(lib);
            Ok(())
        }
    }

    #[cfg(all(feature = "dynamic-loading", not(target_family = "wasm")))]
    #[derive(Debug)]
    pub enum DynamicLoadError {
        LoadFailed(libloading::Error),
        SymbolNotFound(libloading::Error),
    }

    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

    fn with_registry<T>(f: impl FnOnce(&mut Registry) -> T) -> T {
        let lock = REGISTRY.get_or_init(|| Mutex::new(Registry::new()));
        let mut guard = lock.lock().unwrap();
        f(&mut guard)
    }

    /// Register a new SectionFactory. Called by plugins.
    pub fn register_section(factory: &'static dyn SectionFactory) {
        with_registry(|reg| {
            reg.section_factories.push(factory);
        });
    }

    /// Register a new Tag plugin.
    pub fn register_tag(plugin: &'static dyn Tag) {
        let key = plugin.name();
        with_registry(|reg| {
            reg.tags.insert(key, plugin);
        });
    }

    /// Load dynamic plugin library (desktop only)
    #[cfg(all(feature = "dynamic-loading", not(target_family = "wasm")))]
    pub fn load_plugin(path: &std::path::Path) -> Result<(), DynamicLoadError> {
        with_registry(|reg| reg.load_plugin_library(path))
    }

    /// Iterate over all registered SectionFactories (snapshot copy).
    pub fn section_factories() -> Vec<&'static dyn SectionFactory> {
        with_registry(|reg| reg.section_factories.clone())
    }

    /// Lookup a tag plugin by name.
    pub fn get_tag(name: &str) -> Option<&'static dyn Tag> {
        with_registry(|reg| reg.tags.get(name).copied())
    }

    /// Get all registered tag names
    pub fn get_all_tag_names() -> Vec<&'static str> {
        with_registry(|reg| reg.tags.keys().copied().collect())
    }
}

#[cfg(feature = "std")]
pub use registry::*;

#[cfg(not(feature = "std"))]
compile_error!(
    "Plugin registry currently requires 'std' feature; support for no_std will be added later."
);
