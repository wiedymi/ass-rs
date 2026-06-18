//! Alignment types for subtitle positioning

/// Alignment types for subtitle positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    // ASS alignments (numpad layout)
    /// Bottom left alignment
    BottomLeft = 1,
    /// Bottom center alignment
    BottomCenter = 2,
    /// Bottom right alignment
    BottomRight = 3,
    /// Middle left alignment
    MiddleLeft = 4,
    /// Center alignment
    Center = 5,
    /// Middle right alignment
    MiddleRight = 6,
    /// Top left alignment
    TopLeft = 7,
    /// Top center alignment
    TopCenter = 8,
    /// Top right alignment
    TopRight = 9,
}

impl Alignment {
    /// Convert from numeric alignment value
    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::BottomLeft,
            2 => Self::BottomCenter,
            3 => Self::BottomRight,
            4 => Self::MiddleLeft,
            5 => Self::Center,
            6 => Self::MiddleRight,
            7 => Self::TopLeft,
            8 => Self::TopCenter,
            9 => Self::TopRight,
            _ => Self::BottomCenter, // Default
        }
    }

    /// Check if this is a legacy SSA alignment value
    pub fn from_ssa_legacy(value: u8) -> Self {
        // SSA legacy: 1=left, 2=center, 3=right
        // +0=sub(bottom), +4=title(middle), +8=top, +128=mid-title
        match value & 3 {
            1 => {
                // Left alignment
                match value & 12 {
                    0 => Self::BottomLeft,
                    4 => Self::MiddleLeft,
                    8 | 12 => Self::TopLeft,
                    _ => Self::BottomLeft,
                }
            }
            2 => {
                // Center alignment
                match value & 12 {
                    0 => Self::BottomCenter,
                    4 => Self::Center,
                    8 | 12 => Self::TopCenter,
                    _ => Self::BottomCenter,
                }
            }
            3 => {
                // Right alignment
                match value & 12 {
                    0 => Self::BottomRight,
                    4 => Self::MiddleRight,
                    8 | 12 => Self::TopRight,
                    _ => Self::BottomRight,
                }
            }
            _ => Self::BottomCenter,
        }
    }
}
