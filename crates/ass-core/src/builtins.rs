//! Built-in section plugins for common ASS sections.

use crate::plugin::*;
use crate::tokenizer::AssToken;
use std::sync::LazyLock;

// -------------------------------------------------------------------------------------------------
// Script Info
// -------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct ScriptInfo {
    entries: Vec<(String, String)>,
}

impl Section for ScriptInfo {
    fn kind(&self) -> SectionKind {
        SectionKind::ScriptInfo
    }

    fn parse_token(&mut self, token: &AssToken, src: &[u8]) {
        if let AssToken::KeyValue { key, value } = token {
            if let (Some(k), Some(v)) = (key.as_str(src), value.as_str(src)) {
                self.entries.push((k.to_owned(), v.to_owned()));
            }
        }
    }

    fn serialize(&self, dst: &mut String) {
        dst.push_str("[Script Info]\n");
        for (k, v) in &self.entries {
            dst.push_str(k);
            dst.push_str(": ");
            dst.push_str(v);
            dst.push('\n');
        }
    }
}

pub struct ScriptInfoFactory;
impl SectionFactory for ScriptInfoFactory {
    fn create(&self) -> Box<dyn Section> {
        Box::new(ScriptInfo::default())
    }
}

// -------------------------------------------------------------------------------------------------
// V4+ Styles
// -------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct V4Styles {
    raw_lines: Vec<String>,
}

impl Section for V4Styles {
    fn kind(&self) -> SectionKind {
        SectionKind::V4Styles
    }

    fn parse_token(&mut self, token: &AssToken, src: &[u8]) {
        match token {
            AssToken::Raw { span } => {
                if let Some(s) = span.as_str(src) {
                    self.raw_lines.push(s.to_owned());
                }
            }
            AssToken::KeyValue { key, value } => {
                // Handle lines like "Format:" and "Style:" which contain colons
                if let (Some(key_str), Some(value_str)) = (key.as_str(src), value.as_str(src)) {
                    let full_line = format!("{key_str}: {value_str}");
                    self.raw_lines.push(full_line);
                }
            }
            _ => {}
        }
    }

    fn serialize(&self, dst: &mut String) {
        dst.push_str("[V4+ Styles]\n");
        for line in &self.raw_lines {
            dst.push_str(line);
            dst.push('\n');
        }
    }
}

pub struct V4StylesFactory;
impl SectionFactory for V4StylesFactory {
    fn create(&self) -> Box<dyn Section> {
        Box::new(V4Styles::default())
    }
}

// -------------------------------------------------------------------------------------------------
// Events
// -------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Events {
    raw_lines: Vec<String>,
}

impl Section for Events {
    fn kind(&self) -> SectionKind {
        SectionKind::Events
    }

    fn parse_token(&mut self, token: &AssToken, src: &[u8]) {
        match token {
            AssToken::Raw { span } => {
                if let Some(s) = span.as_str(src) {
                    self.raw_lines.push(s.to_owned());
                }
            }
            AssToken::KeyValue { key, value } => {
                // Handle lines like "Format:" and "Dialogue:" which contain colons
                if let (Some(key_str), Some(value_str)) = (key.as_str(src), value.as_str(src)) {
                    let full_line = format!("{key_str}: {value_str}");
                    self.raw_lines.push(full_line);
                }
            }
            _ => {
                // Ignore other token types for now
            }
        }
    }

    fn serialize(&self, dst: &mut String) {
        dst.push_str("[Events]\n");
        for line in &self.raw_lines {
            dst.push_str(line);
            dst.push('\n');
        }
    }
}

pub struct EventsFactory;
impl SectionFactory for EventsFactory {
    fn create(&self) -> Box<dyn Section> {
        Box::new(Events::default())
    }
}

// -------------------------------------------------------------------------------------------------
// Registration helper
// -------------------------------------------------------------------------------------------------

/// Register all built-in section factories with the global registry.
/// Should be called once at startup (e.g., from library `init`).
#[cfg(feature = "std")]
pub fn register_builtins() {
    use crate::plugin::{register_section, register_tag};

    // We leak the factories intentionally to obtain 'static lifetime.
    static SCRIPT_INFO_FACTORY: ScriptInfoFactory = ScriptInfoFactory;
    static V4_STYLES_FACTORY: V4StylesFactory = V4StylesFactory;
    static EVENTS_FACTORY: EventsFactory = EventsFactory;

    register_section(&SCRIPT_INFO_FACTORY);
    register_section(&V4_STYLES_FACTORY);
    register_section(&EVENTS_FACTORY);

    // Register simple tag plugins
    static BOLD_TAG: BoldTag = BoldTag;
    static ITALIC_TAG: ItalicTag = ItalicTag;
    static UNDERLINE_TAG: UnderlineTag = UnderlineTag;
    static STRIKETHROUGH_TAG: StrikethroughTag = StrikethroughTag;
    static COLOR_TAG: ColorTag = ColorTag;
    static ALPHA_TAG: AlphaTag = AlphaTag;
    static FONTSIZE_TAG: FontSizeTag = FontSizeTag;
    static FONTNAME_TAG: FontNameTag = FontNameTag;
    static FONTSCALEX_TAG: FontScaleXTag = FontScaleXTag;
    static FONTSCALEY_TAG: FontScaleYTag = FontScaleYTag;
    static BORDER_TAG: BorderTag = BorderTag;
    static SHADOW_TAG: ShadowTag = ShadowTag;
    static BLUR_TAG: BlurTag = BlurTag;
    static POS_TAG: PosTag = PosTag;
    static MOVE_TAG: MoveTag = MoveTag;
    static ALIGN_TAG: AlignTag = AlignTag;
    static ROTZ_TAG: RotZTag = RotZTag;
    static ROTX_TAG: RotXTag = RotXTag;
    static ROTY_TAG: RotYTag = RotYTag;
    static FADE_TAG: FadeTag = FadeTag;
    static CLIP_TAG: ClipTag = ClipTag;
    static RESET_TAG: ResetTag = ResetTag;
    static TRANSFORM_TAG: TransformTag = TransformTag;
    static KARAOKE_TAG: KaraokeTag = KaraokeTag;
    static KARAOKE_FILL_TAG: KaraokeFillTag = KaraokeFillTag;
    static KARAOKE_OUTLINE_TAG: KaraokeOutlineTag = KaraokeOutlineTag;

    register_tag(&BOLD_TAG);
    register_tag(&ITALIC_TAG);
    register_tag(&UNDERLINE_TAG);
    register_tag(&STRIKETHROUGH_TAG);
    register_tag(&COLOR_TAG);
    register_tag(&ALPHA_TAG);
    register_tag(&FONTSIZE_TAG);
    register_tag(&FONTNAME_TAG);
    register_tag(&FONTSCALEX_TAG);
    register_tag(&FONTSCALEY_TAG);
    register_tag(&BORDER_TAG);
    register_tag(&SHADOW_TAG);
    register_tag(&BLUR_TAG);
    register_tag(&POS_TAG);
    register_tag(&MOVE_TAG);
    register_tag(&ALIGN_TAG);
    register_tag(&ROTZ_TAG);
    register_tag(&ROTX_TAG);
    register_tag(&ROTY_TAG);
    register_tag(&FADE_TAG);
    register_tag(&CLIP_TAG);
    register_tag(&RESET_TAG);
    register_tag(&TRANSFORM_TAG);
    register_tag(&KARAOKE_TAG);
    register_tag(&KARAOKE_FILL_TAG);
    register_tag(&KARAOKE_OUTLINE_TAG);
}

// -------------------------------------------------------------------------------------------------
// Basic built-in tag: \b (bold) -- placeholder
// -------------------------------------------------------------------------------------------------

pub struct BoldTag;

impl crate::plugin::Tag for BoldTag {
    fn name(&self) -> &'static str {
        "b"
    }
}

pub struct ItalicTag;

impl crate::plugin::Tag for ItalicTag {
    fn name(&self) -> &'static str {
        "i"
    }
}

pub struct UnderlineTag;
impl crate::plugin::Tag for UnderlineTag {
    fn name(&self) -> &'static str {
        "u"
    }
}

pub struct StrikethroughTag;
impl crate::plugin::Tag for StrikethroughTag {
    fn name(&self) -> &'static str {
        "s"
    }
}

pub struct ColorTag;
impl crate::plugin::Tag for ColorTag {
    fn name(&self) -> &'static str {
        "c"
    }
}

pub struct AlphaTag;
impl crate::plugin::Tag for AlphaTag {
    fn name(&self) -> &'static str {
        "alpha"
    }
}

pub struct FontSizeTag;
impl crate::plugin::Tag for FontSizeTag {
    fn name(&self) -> &'static str {
        "fs"
    }
}

pub struct FontNameTag;
impl crate::plugin::Tag for FontNameTag {
    fn name(&self) -> &'static str {
        "fn"
    }
}

pub struct FontScaleXTag;
impl crate::plugin::Tag for FontScaleXTag {
    fn name(&self) -> &'static str {
        "fscx"
    }
}

pub struct FontScaleYTag;
impl crate::plugin::Tag for FontScaleYTag {
    fn name(&self) -> &'static str {
        "fscy"
    }
}

pub struct BorderTag;
impl crate::plugin::Tag for BorderTag {
    fn name(&self) -> &'static str {
        "bord"
    }
}

pub struct ShadowTag;
impl crate::plugin::Tag for ShadowTag {
    fn name(&self) -> &'static str {
        "shad"
    }
}

pub struct BlurTag;
impl crate::plugin::Tag for BlurTag {
    fn name(&self) -> &'static str {
        "be"
    }
}

pub struct PosTag;
impl crate::plugin::Tag for PosTag {
    fn name(&self) -> &'static str {
        "pos"
    }
}

pub struct MoveTag;
impl crate::plugin::Tag for MoveTag {
    fn name(&self) -> &'static str {
        "move"
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        static MOVE_ARGS: LazyLock<Vec<crate::plugin::TagArgumentType>> = LazyLock::new(|| {
            vec![
                crate::plugin::TagArgumentType::Float, // x1
                crate::plugin::TagArgumentType::Float, // y1
                crate::plugin::TagArgumentType::Float, // x2
                crate::plugin::TagArgumentType::Float, // y2
                crate::plugin::TagArgumentType::Optional(Box::new(
                    crate::plugin::TagArgumentType::Float,
                )), // t1
                crate::plugin::TagArgumentType::Optional(Box::new(
                    crate::plugin::TagArgumentType::Float,
                )), // t2
            ]
        });
        &MOVE_ARGS
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        if parts.len() < 4 || parts.len() > 6 {
            return Err(crate::plugin::TagParseError::WrongArgumentCount);
        }

        let x1: f32 = parts[0].parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;
        let y1: f32 = parts[1].parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;
        let x2: f32 = parts[2].parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;
        let y2: f32 = parts[3].parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;

        let mut args_out = vec![
            crate::plugin::TagArgument::Position(x1, y1),
            crate::plugin::TagArgument::Position(x2, y2),
        ];

        if parts.len() >= 6 {
            let t1: f32 = parts[4].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;
            let t2: f32 = parts[5].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;
            args_out.push(crate::plugin::TagArgument::Float(t1));
            args_out.push(crate::plugin::TagArgument::Float(t2));
        }

        Ok(args_out)
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(
        &self,
        args: &[crate::plugin::TagArgument],
        state: &mut crate::plugin::AnimationState,
    ) -> Result<(), crate::plugin::TagApplicationError> {
        if args.len() < 2 {
            return Err(crate::plugin::TagApplicationError::InvalidState);
        }

        if let (
            crate::plugin::TagArgument::Position(x1, y1),
            crate::plugin::TagArgument::Position(x2, y2),
        ) = (&args[0], &args[1])
        {
            let (start_time, end_time) = if args.len() >= 4 {
                if let (
                    crate::plugin::TagArgument::Float(t1),
                    crate::plugin::TagArgument::Float(t2),
                ) = (&args[2], &args[3])
                {
                    (*t1 as f64 / 1000.0, *t2 as f64 / 1000.0)
                } else {
                    (0.0, 1.0)
                }
            } else {
                (0.0, 1.0) // Default animation duration
            };

            let animation = crate::plugin::ActiveAnimation {
                tag_name: "pos".to_string(),
                start_time,
                end_time,
                start_value: crate::plugin::TagArgument::Position(*x1, *y1),
                end_value: crate::plugin::TagArgument::Position(*x2, *y2),
                mode: crate::plugin::AnimationMode::Linear,
            };

            state.add_animation(animation);
        }

        Ok(())
    }
}

// Alignment \an (digital 1..9)
pub struct AlignTag;
impl crate::plugin::Tag for AlignTag {
    fn name(&self) -> &'static str {
        "an"
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        if args.is_empty() {
            return Ok(vec![]);
        }

        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let alignment: i32 = args_str.trim().parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Integer)
        })?;

        if !(1..=9).contains(&alignment) {
            return Err(crate::plugin::TagParseError::InvalidArguments);
        }

        Ok(vec![crate::plugin::TagArgument::Integer(alignment)])
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[crate::plugin::TagArgumentType::Integer]
    }
}

// Rotation Z \frz
pub struct RotZTag;
impl crate::plugin::Tag for RotZTag {
    fn name(&self) -> &'static str {
        "frz"
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        if args.is_empty() {
            return Ok(vec![crate::plugin::TagArgument::Float(0.0)]);
        }

        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let rotation: f32 = args_str.trim().parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;

        Ok(vec![crate::plugin::TagArgument::Float(rotation)])
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[crate::plugin::TagArgumentType::Float]
    }
}

// Rotation X \frx
pub struct RotXTag;
impl crate::plugin::Tag for RotXTag {
    fn name(&self) -> &'static str {
        "frx"
    }
}

// Rotation Y \fry
pub struct RotYTag;
impl crate::plugin::Tag for RotYTag {
    fn name(&self) -> &'static str {
        "fry"
    }
}

// Fade \fad(a,b)
pub struct FadeTag;
impl crate::plugin::Tag for FadeTag {
    fn name(&self) -> &'static str {
        "fad"
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[
            crate::plugin::TagArgumentType::Float, // fade in time
            crate::plugin::TagArgumentType::Float, // fade out time
        ]
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        if parts.len() == 2 {
            // \fad(t1,t2) format
            let t1: f32 = parts[0].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;
            let t2: f32 = parts[1].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;

            Ok(vec![
                crate::plugin::TagArgument::Float(t1),
                crate::plugin::TagArgument::Float(t2),
            ])
        } else if parts.len() == 7 {
            // \fade(a1,a2,a3,t1,t2,t3,t4) format
            let a1: i32 = parts[0].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Integer)
            })?;
            let a2: i32 = parts[1].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Integer)
            })?;
            let a3: i32 = parts[2].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Integer)
            })?;
            let t1: f32 = parts[3].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;
            let t2: f32 = parts[4].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;
            let t3: f32 = parts[5].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;
            let t4: f32 = parts[6].parse().map_err(|_| {
                crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
            })?;

            Ok(vec![
                crate::plugin::TagArgument::Integer(a1),
                crate::plugin::TagArgument::Integer(a2),
                crate::plugin::TagArgument::Integer(a3),
                crate::plugin::TagArgument::Float(t1),
                crate::plugin::TagArgument::Float(t2),
                crate::plugin::TagArgument::Float(t3),
                crate::plugin::TagArgument::Float(t4),
            ])
        } else {
            Err(crate::plugin::TagParseError::WrongArgumentCount)
        }
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(
        &self,
        args: &[crate::plugin::TagArgument],
        state: &mut crate::plugin::AnimationState,
    ) -> Result<(), crate::plugin::TagApplicationError> {
        if args.len() == 2 {
            // Simple fade in/out
            if let (
                crate::plugin::TagArgument::Float(fade_in),
                crate::plugin::TagArgument::Float(fade_out),
            ) = (&args[0], &args[1])
            {
                let fade_in_anim = crate::plugin::ActiveAnimation {
                    tag_name: "alpha_fade_in".to_string(),
                    start_time: 0.0,
                    end_time: *fade_in as f64 / 1000.0,
                    start_value: crate::plugin::TagArgument::Integer(255),
                    end_value: crate::plugin::TagArgument::Integer(0),
                    mode: crate::plugin::AnimationMode::Linear,
                };

                let fade_out_anim = crate::plugin::ActiveAnimation {
                    tag_name: "alpha_fade_out".to_string(),
                    start_time: state.current_time - (*fade_out as f64 / 1000.0),
                    end_time: state.current_time,
                    start_value: crate::plugin::TagArgument::Integer(0),
                    end_value: crate::plugin::TagArgument::Integer(255),
                    mode: crate::plugin::AnimationMode::Linear,
                };

                state.add_animation(fade_in_anim);
                state.add_animation(fade_out_anim);
            }
        } else if args.len() == 7 {
            // Complex fade with alpha values and timing
            // Implementation would handle the complex fade animation
        }

        Ok(())
    }
}

// Clip \clip
pub struct ClipTag;
impl crate::plugin::Tag for ClipTag {
    fn name(&self) -> &'static str {
        "clip"
    }
}

// Reset \r
pub struct ResetTag;
impl crate::plugin::Tag for ResetTag {
    fn name(&self) -> &'static str {
        "r"
    }
}

// -------------------------------------------------------------------------------------------------
// Advanced Animation Tags
// -------------------------------------------------------------------------------------------------

/// Transform tag for complex animations (\t)
pub struct TransformTag;
impl crate::plugin::Tag for TransformTag {
    fn name(&self) -> &'static str {
        "t"
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[crate::plugin::TagArgumentType::String]
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        Ok(vec![crate::plugin::TagArgument::String(
            args_str.to_string(),
        )])
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(
        &self,
        _args: &[crate::plugin::TagArgument],
        _state: &mut crate::plugin::AnimationState,
    ) -> Result<(), crate::plugin::TagApplicationError> {
        // Transform parsing and application would be implemented here
        Ok(())
    }
}

/// Karaoke effect tag (\k, \kf, \ko)
pub struct KaraokeTag;
impl crate::plugin::Tag for KaraokeTag {
    fn name(&self) -> &'static str {
        "k"
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[crate::plugin::TagArgumentType::Float]
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let duration: f32 = args_str.trim().parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;
        Ok(vec![crate::plugin::TagArgument::Float(duration)])
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(
        &self,
        args: &[crate::plugin::TagArgument],
        state: &mut crate::plugin::AnimationState,
    ) -> Result<(), crate::plugin::TagApplicationError> {
        if let Some(crate::plugin::TagArgument::Float(duration)) = args.first() {
            let animation = crate::plugin::ActiveAnimation {
                tag_name: "karaoke".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + (*duration as f64 / 100.0), // centiseconds to seconds
                start_value: crate::plugin::TagArgument::Float(0.0),
                end_value: crate::plugin::TagArgument::Float(1.0),
                mode: crate::plugin::AnimationMode::Linear,
            };

            state.add_animation(animation);
            Ok(())
        } else {
            Err(crate::plugin::TagApplicationError::InvalidState)
        }
    }
}

pub struct KaraokeFillTag;
impl crate::plugin::Tag for KaraokeFillTag {
    fn name(&self) -> &'static str {
        "kf"
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[crate::plugin::TagArgumentType::Float]
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let duration: f32 = args_str.trim().parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;
        Ok(vec![crate::plugin::TagArgument::Float(duration)])
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(
        &self,
        args: &[crate::plugin::TagArgument],
        state: &mut crate::plugin::AnimationState,
    ) -> Result<(), crate::plugin::TagApplicationError> {
        if let Some(crate::plugin::TagArgument::Float(duration)) = args.first() {
            let animation = crate::plugin::ActiveAnimation {
                tag_name: "karaoke_fill".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + (*duration as f64 / 100.0),
                start_value: crate::plugin::TagArgument::Float(0.0),
                end_value: crate::plugin::TagArgument::Float(1.0),
                mode: crate::plugin::AnimationMode::Linear,
            };

            state.add_animation(animation);
            Ok(())
        } else {
            Err(crate::plugin::TagApplicationError::InvalidState)
        }
    }
}

pub struct KaraokeOutlineTag;
impl crate::plugin::Tag for KaraokeOutlineTag {
    fn name(&self) -> &'static str {
        "ko"
    }

    fn expected_args(&self) -> &[crate::plugin::TagArgumentType] {
        &[crate::plugin::TagArgumentType::Float]
    }

    fn parse_args(
        &self,
        args: &[u8],
    ) -> Result<Vec<crate::plugin::TagArgument>, crate::plugin::TagParseError> {
        let args_str = std::str::from_utf8(args)
            .map_err(|_| crate::plugin::TagParseError::InvalidArguments)?;
        let duration: f32 = args_str.trim().parse().map_err(|_| {
            crate::plugin::TagParseError::TypeMismatch(crate::plugin::TagArgumentType::Float)
        })?;
        Ok(vec![crate::plugin::TagArgument::Float(duration)])
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(
        &self,
        args: &[crate::plugin::TagArgument],
        state: &mut crate::plugin::AnimationState,
    ) -> Result<(), crate::plugin::TagApplicationError> {
        if let Some(crate::plugin::TagArgument::Float(duration)) = args.first() {
            let animation = crate::plugin::ActiveAnimation {
                tag_name: "karaoke_outline".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + (*duration as f64 / 100.0),
                start_value: crate::plugin::TagArgument::Float(0.0),
                end_value: crate::plugin::TagArgument::Float(1.0),
                mode: crate::plugin::AnimationMode::Linear,
            };

            state.add_animation(animation);
            Ok(())
        } else {
            Err(crate::plugin::TagApplicationError::InvalidState)
        }
    }
}

// Built-in utility functions for ASS script processing
//
// This module provides common utility functions for time parsing,
// color manipulation, string processing, and other ASS-related operations.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Parse time string in ASS format (H:MM:SS.CC) to milliseconds
pub fn parse_time(time_str: &str) -> Result<u64, String> {
    if time_str.is_empty() {
        return Err("Empty time string".to_string());
    }

    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid time format".to_string());
    }

    let hours: u64 = parts[0].parse().map_err(|_| "Invalid hours")?;
    let minutes: u64 = parts[1].parse().map_err(|_| "Invalid minutes")?;

    let sec_parts: Vec<&str> = parts[2].split('.').collect();
    if sec_parts.len() != 2 {
        return Err("Invalid seconds format".to_string());
    }

    let seconds: u64 = sec_parts[0].parse().map_err(|_| "Invalid seconds")?;
    let centiseconds: u64 = sec_parts[1].parse().map_err(|_| "Invalid centiseconds")?;

    if minutes >= 60 || seconds >= 60 || centiseconds >= 100 {
        return Err("Time component out of range".to_string());
    }

    Ok((hours * 3600 + minutes * 60 + seconds) * 1000 + centiseconds * 10)
}

/// Format milliseconds to ASS time format (H:MM:SS.CC)
pub fn format_time(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1000;
    let centiseconds = (milliseconds % 1000) / 10;

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}")
}

/// Parse color string in ASS format (&HBBGGRR&) to RGB value
pub fn parse_color(color_str: &str) -> Result<u32, String> {
    if color_str.is_empty() {
        return Err("Empty color string".to_string());
    }

    if !color_str.starts_with("&H") || !color_str.ends_with('&') {
        return Err("Invalid color format".to_string());
    }

    let hex_part = &color_str[2..color_str.len() - 1];
    if hex_part.len() != 6 && hex_part.len() != 8 {
        return Err("Invalid color length".to_string());
    }

    let value = u32::from_str_radix(hex_part, 16).map_err(|_| "Invalid hexadecimal color")?;

    // ASS colors are in BBGGRR format, convert to RRGGBB
    let b = (value >> 16) & 0xFF;
    let g = (value >> 8) & 0xFF;
    let r = value & 0xFF;

    Ok((r << 16) | (g << 8) | b)
}

/// Format RGB color to ASS format (&HBBGGRR&)
pub fn format_color(rgb: u32) -> String {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;

    // Convert to ASS BBGGRR format
    let ass_color = (b << 16) | (g << 8) | r;
    format!("&H{ass_color:06X}&")
}

/// Trim whitespace from both ends of a string
pub fn trim_whitespace(text: &str) -> &str {
    text.trim()
}

/// Replace all occurrences of a pattern in text
pub fn replace_text(text: &str, pattern: &str, replacement: &str) -> String {
    if pattern.is_empty() {
        return text.to_string();
    }
    text.replace(pattern, replacement)
}

/// Convert text to uppercase
pub fn to_uppercase(text: &str) -> String {
    text.to_uppercase()
}

/// Convert text to lowercase
pub fn to_lowercase(text: &str) -> String {
    text.to_lowercase()
}

/// Linear interpolation between two values
pub fn lerp(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}

/// Convert degrees to radians
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * core::f64::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / core::f64::consts::PI
}

/// Clamp value between min and max
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value.is_nan() {
        value
    } else if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Parse tag parameters from a string
pub fn parse_tag_parameters(params: &str) -> Result<Vec<String>, String> {
    if params.is_empty() {
        return Err("Empty parameters".to_string());
    }

    Ok(params.split(',').map(|s| s.trim().to_string()).collect())
}

/// Validate ASS timestamp format
pub fn is_valid_timestamp(timestamp: &str) -> bool {
    parse_time(timestamp).is_ok()
}

/// Validate tag syntax
pub fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty() && tag.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Convert tag to standard format
pub fn format_tag(tag: &str) -> String {
    tag.trim().to_lowercase()
}

/// Get color components from RGB value
pub fn get_color_components(rgb: u32) -> (u8, u8, u8) {
    let r = ((rgb >> 16) & 0xFF) as u8;
    let g = ((rgb >> 8) & 0xFF) as u8;
    let b = (rgb & 0xFF) as u8;
    (r, g, b)
}

/// Create RGB color from components
pub fn create_color(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Validate script section name
pub fn is_valid_section_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(|c: char| c.is_control() || c == '[' || c == ']')
}

/// Normalize line endings to \n
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Escape special characters for ASS format
pub fn escape_ass_text(text: &str) -> String {
    text.replace('\\', r"\\")
        .replace('{', r"\{")
        .replace('}', r"\}")
}

/// Unescape ASS text
pub fn unescape_ass_text(text: &str) -> String {
    text.replace(r"\\", "\\")
        .replace(r"\{", "{")
        .replace(r"\}", "}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_parsing() {
        assert_eq!(parse_time("0:01:30.50").unwrap(), 90500);
        assert_eq!(parse_time("1:00:00.00").unwrap(), 3600000);
        assert!(parse_time("").is_err());
        assert!(parse_time("invalid").is_err());
    }

    #[test]
    fn test_color_parsing() {
        assert_eq!(parse_color("&H0000FF&").unwrap(), 0xFF0000); // Red
        assert_eq!(parse_color("&H00FF00&").unwrap(), 0x00FF00); // Green
        assert!(parse_color("").is_err());
        assert!(parse_color("invalid").is_err());
    }

    #[test]
    fn test_string_functions() {
        assert_eq!(trim_whitespace("  hello  "), "hello");
        assert_eq!(replace_text("hello world", "world", "Rust"), "hello Rust");
        assert_eq!(to_uppercase("hello"), "HELLO");
        assert_eq!(to_lowercase("WORLD"), "world");
    }
}
