//! Shared karaoke tag type definitions used across karaoke commands.

/// ASS karaoke tag types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KaraokeType {
    /// \k - standard karaoke (highlights during duration)
    Standard,
    /// \kf or \K - fill karaoke (sweeps from left to right)
    Fill,
    /// \ko - outline karaoke (outline changes during duration)
    Outline,
    /// \kt - transition karaoke (for advanced effects)
    Transition,
}

impl KaraokeType {
    /// Get the ASS tag string for this karaoke type
    pub fn tag_string(self) -> &'static str {
        match self {
            KaraokeType::Standard => "k",
            KaraokeType::Fill => "kf",
            KaraokeType::Outline => "ko",
            KaraokeType::Transition => "kt",
        }
    }
}
