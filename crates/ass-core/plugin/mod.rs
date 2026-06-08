//! Plugin system for extending ASS parsing and rendering capabilities.
//!
//! This module provides a trait-based extension system allowing custom tag handlers,
//! section processors, and rendering backends to be registered at runtime. Designed
//! for zero-allocation hot paths and efficient lookup via optimized hash maps.
//!
//! ## Architecture
//!
//! - **`TagHandler`**: Process custom override tags (e.g., `{\custom}`)
//! - **`SectionProcessor`**: Handle non-standard sections (e.g., `[Aegisub Project]`)
//! - **`ExtensionRegistry`**: Central registry for all extensions
//!
//! ## Example
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, TagHandler, TagResult};
//!
//! struct CustomColorTag;
//!
//! impl TagHandler for CustomColorTag {
//!     fn name(&self) -> &'static str { "customcolor" }
//!
//!     fn process(&self, args: &str) -> TagResult {
//!         // Custom color processing logic
//!         TagResult::Processed
//!     }
//! }
//!
//! let mut registry = ExtensionRegistry::new();
//! registry.register_tag_handler(Box::new(CustomColorTag));
//! ```

pub mod sections;
pub mod tags;

mod error;
mod registry;
mod traits;

#[cfg(test)]
mod tests;

pub use error::{PluginError, Result};
pub use registry::ExtensionRegistry;
pub use traits::{SectionProcessor, SectionResult, TagHandler, TagResult};

pub use sections::aegisub::{
    create_aegisub_processors, AegisubExtradataProcessor, AegisubProjectProcessor,
};
pub use tags::{
    advanced::{create_advanced_handlers, BlurEdgesTagHandler, BorderTagHandler, ShadowTagHandler},
    alignment::{
        create_alignment_handlers, AlignmentTagHandler, NumpadAlignmentTagHandler,
        WrappingStyleTagHandler,
    },
    animation::{
        create_animation_handlers, FadeTagHandler, SimpleFadeTagHandler, TransformTagHandler,
    },
    clipping::{create_clipping_handlers, ClipTagHandler},
    color::{
        create_color_handlers, Alpha1TagHandler, Alpha2TagHandler, Alpha3TagHandler,
        Alpha4TagHandler, AlphaTagHandler, Color1TagHandler, Color2TagHandler, Color3TagHandler,
        Color4TagHandler, PrimaryColorTagHandler,
    },
    font::{create_font_handlers, FontEncodingTagHandler, FontNameTagHandler, FontSizeTagHandler},
    formatting::{
        create_formatting_handlers, BoldTagHandler, ItalicTagHandler, StrikeoutTagHandler,
        UnderlineTagHandler,
    },
    karaoke::{
        create_karaoke_handlers, BasicKaraokeTagHandler, FillKaraokeTagHandler,
        KaraokeTimingTagHandler, OutlineKaraokeTagHandler,
    },
    misc::{create_misc_handlers, OriginTagHandler, ResetTagHandler, ShortRotationTagHandler},
    position::{create_position_handlers, MoveTagHandler, PositionTagHandler},
    special::{
        create_special_handlers, HardLineBreakTagHandler, HardSpaceTagHandler,
        SoftLineBreakTagHandler,
    },
    transform::{
        create_transform_handlers, RotationXTagHandler, RotationYTagHandler, RotationZTagHandler,
        ScaleXTagHandler, ScaleYTagHandler, ShearXTagHandler, ShearYTagHandler, SpacingTagHandler,
    },
};
