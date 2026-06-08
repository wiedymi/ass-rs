//! Synthetic ASS script generator configuration and constructors.
//!
//! Defines the [`ScriptGenerator`] configuration struct and the
//! [`ComplexityLevel`] enum together with the constructors that build
//! generators for each supported benchmarking scenario.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Synthetic ASS script generator for benchmarking
pub struct ScriptGenerator {
    /// Script title for metadata
    pub title: String,
    /// Number of styles to generate
    pub styles_count: usize,
    /// Number of events to generate
    pub events_count: usize,
    /// Complexity level for generated content
    pub complexity_level: ComplexityLevel,
}

/// Script complexity levels for testing
#[derive(Debug, Clone, Copy)]
pub enum ComplexityLevel {
    /// Simple text with minimal formatting
    Simple,
    /// Moderate formatting and some animations
    Moderate,
    /// Heavy animations, complex styling, karaoke
    Complex,
    /// Extreme complexity to stress-test parser
    Extreme,
    /// Anime subtitles - heavy styling and effects
    AnimeRealistic,
    /// Movie subtitles - simple, timing-focused
    MovieRealistic,
    /// Karaoke files - complex animations and timing
    KaraokeRealistic,
    /// Sign translations - positioning and styling
    SignRealistic,
    /// Educational content - long dialogues and formatting
    EducationalRealistic,
}

impl ScriptGenerator {
    /// Create generator for simple scripts
    #[must_use]
    pub fn simple(events_count: usize) -> Self {
        Self {
            title: "Simple Benchmark Script".to_string(),
            styles_count: 1,
            events_count,
            complexity_level: ComplexityLevel::Simple,
        }
    }

    /// Create generator for moderate complexity scripts
    #[must_use]
    pub fn moderate(events_count: usize) -> Self {
        Self {
            title: "Moderate Benchmark Script".to_string(),
            styles_count: 5,
            events_count,
            complexity_level: ComplexityLevel::Moderate,
        }
    }

    /// Create generator for complex scripts
    #[must_use]
    pub fn complex(events_count: usize) -> Self {
        Self {
            title: "Complex Benchmark Script".to_string(),
            styles_count: 10,
            events_count,
            complexity_level: ComplexityLevel::Complex,
        }
    }

    /// Create generator for extreme complexity scripts
    #[must_use]
    pub fn extreme(events_count: usize) -> Self {
        Self {
            title: "Extreme Benchmark Script".to_string(),
            styles_count: 20,
            events_count,
            complexity_level: ComplexityLevel::Extreme,
        }
    }

    /// Create generator for anime-style subtitles
    #[must_use]
    pub fn anime_realistic(events_count: usize) -> Self {
        Self {
            title: "Anime Subtitles".to_string(),
            styles_count: 15,
            events_count,
            complexity_level: ComplexityLevel::AnimeRealistic,
        }
    }

    /// Create generator for movie subtitles
    #[must_use]
    pub fn movie_realistic(events_count: usize) -> Self {
        Self {
            title: "Movie Subtitles".to_string(),
            styles_count: 3,
            events_count,
            complexity_level: ComplexityLevel::MovieRealistic,
        }
    }

    /// Create generator for karaoke files
    #[must_use]
    pub fn karaoke_realistic(events_count: usize) -> Self {
        Self {
            title: "Karaoke Script".to_string(),
            styles_count: 8,
            events_count,
            complexity_level: ComplexityLevel::KaraokeRealistic,
        }
    }

    /// Create generator for sign translation subtitles
    #[must_use]
    pub fn sign_realistic(events_count: usize) -> Self {
        Self {
            title: "Sign Translation".to_string(),
            styles_count: 12,
            events_count,
            complexity_level: ComplexityLevel::SignRealistic,
        }
    }

    /// Create generator for educational content
    #[must_use]
    pub fn educational_realistic(events_count: usize) -> Self {
        Self {
            title: "Educational Content".to_string(),
            styles_count: 6,
            events_count,
            complexity_level: ComplexityLevel::EducationalRealistic,
        }
    }
}
