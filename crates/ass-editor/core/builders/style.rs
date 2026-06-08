//! Style builder type and constructors.

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Builder for creating ASS styles with fluent API
#[derive(Debug, Default, Clone)]
pub struct StyleBuilder {
    pub(super) name: Option<String>,
    pub(super) fontname: Option<String>,
    pub(super) fontsize: Option<u32>,
    pub(super) primary_colour: Option<String>,
    pub(super) secondary_colour: Option<String>,
    pub(super) outline_colour: Option<String>,
    pub(super) back_colour: Option<String>,
    pub(super) bold: Option<bool>,
    pub(super) italic: Option<bool>,
    pub(super) underline: Option<bool>,
    pub(super) strikeout: Option<bool>,
    pub(super) scale_x: Option<f32>,
    pub(super) scale_y: Option<f32>,
    pub(super) spacing: Option<f32>,
    pub(super) angle: Option<f32>,
    pub(super) border_style: Option<u32>,
    pub(super) outline: Option<f32>,
    pub(super) shadow: Option<f32>,
    pub(super) alignment: Option<u32>,
    pub(super) margin_l: Option<u32>,
    pub(super) margin_r: Option<u32>,
    pub(super) margin_v: Option<u32>,
    pub(super) margin_t: Option<u32>,
    pub(super) margin_b: Option<u32>,
    pub(super) encoding: Option<u32>,
    pub(super) alpha_level: Option<u32>,
    pub(super) relative_to: Option<String>,
}

impl StyleBuilder {
    /// Create a new style builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a style builder with default values
    pub fn default_style() -> Self {
        Self {
            fontname: Some("Arial".to_string()),
            fontsize: Some(20),
            primary_colour: Some("&Hffffff".to_string()),
            secondary_colour: Some("&Hff0000".to_string()),
            outline_colour: Some("&H0".to_string()),
            back_colour: Some("&H0".to_string()),
            bold: Some(false),
            italic: Some(false),
            underline: Some(false),
            strikeout: Some(false),
            scale_x: Some(100.0),
            scale_y: Some(100.0),
            spacing: Some(0.0),
            angle: Some(0.0),
            border_style: Some(1),
            outline: Some(2.0),
            shadow: Some(0.0),
            alignment: Some(2),
            margin_l: Some(10),
            margin_r: Some(10),
            margin_v: Some(10),
            encoding: Some(1),
            ..Self::default()
        }
    }
}
