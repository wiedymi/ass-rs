//! Common data structures for rendering.

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct StyleState {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: u32, // RRGGBB
    pub alpha: f32, // 0.0 - 1.0
    pub font_size: f32,
    pub font_name: Option<&'static str>,
    pub font_scale_x: f32,        // percentage (100.0 = normal)
    pub font_scale_y: f32,        // percentage (100.0 = normal)
    pub border_style: u8,         // 0=outline+drop-shadow, 1=opaque box
    pub border_size: f32,         // outline width
    pub shadow_depth: f32,        // shadow offset
    pub blur_edges: f32,          // blur radius
    pub align: u8,                // 1-9 (numpad alignment)
    pub rot_x: f32,               // rotation around X axis (degrees)
    pub rot_y: f32,               // rotation around Y axis (degrees)
    pub rot_z: f32,               // rotation around Z axis (degrees)
    pub fade: Option<(u16, u16)>, // fade in/out milliseconds
    pub clip_rect: Option<ClipRect>,
}

#[derive(Debug, Clone, Copy)]
pub struct ClipRect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Default for StyleState {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            color: 0xFFFFFF, // white
            alpha: 1.0,
            font_size: 32.0,
            font_name: None,
            font_scale_x: 100.0,
            font_scale_y: 100.0,
            border_style: 0,
            border_size: 2.0,
            shadow_depth: 0.0,
            blur_edges: 0.0,
            align: 2, // bottom center
            rot_x: 0.0,
            rot_y: 0.0,
            rot_z: 0.0,
            fade: None,
            clip_rect: None,
        }
    }
}

#[derive(Debug)]
pub struct Segment<'a> {
    pub text: &'a str,
    pub style: StyleState,
}

#[derive(Debug)]
pub struct RenderedLine {
    pub segments: Vec<Segment<'static>>,
    pub alpha: f32, // line-level alpha
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub fade: Option<(u16, u16)>,
    pub align: u8,
    pub pos: Option<Pos>,
    pub movement: Option<Movement>,
}

#[derive(Debug, Clone, Copy)]
pub struct Movement {
    pub start_pos: Pos,
    pub end_pos: Pos,
    pub start_time: f64, // seconds relative to line start
    pub end_time: f64,   // seconds relative to line start
}

#[derive(Debug)]
pub struct Frame {
    pub lines: Vec<RenderedLine>,
}
