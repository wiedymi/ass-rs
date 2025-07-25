//! ASS override tag handlers for the plugin system
//!
//! Provides implementations of the `TagHandler` trait for all standard ASS
//! override tags. Handlers validate tag arguments and process tag operations
//! according to ASS specification compliance.
//!
//! # Modules
//!
//! - [`formatting`] - Basic text formatting tags (bold, italic, underline, strikeout)
//! - [`special`] - Special character tags (\n, \N, \h)
//! - [`font`] - Font control tags (\fn, \fs, \fe)
//! - [`advanced`] - Advanced formatting tags (\bord, \shad, \be)
//! - [`alignment`] - Text alignment tags (\a, \an, \q)
//! - [`karaoke`] - Karaoke timing tags (\k, \kf, \ko, \kt)
//! - [`position`] - Positioning and movement tags (\pos, \move)
//! - [`color`] - Color and alpha channel tags (\c, \alpha, \1-4c, \1-4a)
//! - [`transform`] - Transform and rotation tags (\frz, \frx, \fry, \fscx, \fscy, \fax, \fay, \fsp)
//! - [`animation`] - Animation tags (\t, \fade, \fad)
//! - [`clipping`] - Clipping mask tags (\clip)
//! - [`misc`] - Miscellaneous tags (\r, \fr, \org)
//!
//! # Usage
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, tags::formatting::create_formatting_handlers};
//!
//! let mut registry = ExtensionRegistry::new();
//! for handler in create_formatting_handlers() {
//!     registry.register_tag_handler(handler).unwrap();
//! }
//! ```

pub mod advanced;
pub mod alignment;
pub mod animation;
pub mod clipping;
pub mod color;
pub mod font;
pub mod formatting;
pub mod karaoke;
pub mod misc;
pub mod position;
pub mod special;
pub mod transform;

pub use advanced::{
    create_advanced_handlers, BlurEdgesTagHandler, BorderTagHandler, ShadowTagHandler,
};
pub use alignment::{
    create_alignment_handlers, AlignmentTagHandler, NumpadAlignmentTagHandler,
    WrappingStyleTagHandler,
};
pub use animation::{
    create_animation_handlers, FadeTagHandler, SimpleFadeTagHandler, TransformTagHandler,
};
pub use clipping::{create_clipping_handlers, ClipTagHandler};
pub use color::{
    create_color_handlers, Alpha1TagHandler, Alpha2TagHandler, Alpha3TagHandler, Alpha4TagHandler,
    AlphaTagHandler, Color1TagHandler, Color2TagHandler, Color3TagHandler, Color4TagHandler,
    PrimaryColorTagHandler,
};
pub use font::{
    create_font_handlers, FontEncodingTagHandler, FontNameTagHandler, FontSizeTagHandler,
};
pub use formatting::{
    create_formatting_handlers, BoldTagHandler, ItalicTagHandler, StrikeoutTagHandler,
    UnderlineTagHandler,
};
pub use karaoke::{
    create_karaoke_handlers, BasicKaraokeTagHandler, FillKaraokeTagHandler,
    KaraokeTimingTagHandler, OutlineKaraokeTagHandler,
};
pub use misc::{create_misc_handlers, OriginTagHandler, ResetTagHandler, ShortRotationTagHandler};
pub use position::{create_position_handlers, MoveTagHandler, PositionTagHandler};
pub use special::{
    create_special_handlers, HardLineBreakTagHandler, HardSpaceTagHandler, SoftLineBreakTagHandler,
};
pub use transform::{
    create_transform_handlers, RotationXTagHandler, RotationYTagHandler, RotationZTagHandler,
    ScaleXTagHandler, ScaleYTagHandler, ShearXTagHandler, ShearYTagHandler, SpacingTagHandler,
};
