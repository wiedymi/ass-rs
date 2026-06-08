//! Dialogue text generators for each benchmarking complexity level.
//!
//! Produces the per-event dialogue strings used by the synthetic generator,
//! ranging from plain text to heavily styled anime, karaoke, sign, and
//! educational presets.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    fmt::Write,
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::fmt::Write;

use super::{ComplexityLevel, ScriptGenerator};

impl ScriptGenerator {
    /// Generate dialogue text based on complexity level
    pub(super) fn generate_dialogue_text(&self, event_index: usize) -> String {
        let base_text = format!("This is dialogue line number {}", event_index + 1);

        match self.complexity_level {
            ComplexityLevel::Simple => base_text,
            ComplexityLevel::Moderate => {
                format!(r"{{\b1}}{base_text}{{\b0}} with {{\i1}}some{{\i0}} formatting")
            }
            ComplexityLevel::Complex => {
                format!(
                    r"{{\pos(100,200)\fad(500,500)\b1\i1\c&H00FF00&}}{base_text}{{\b0\i0\c&HFFFFFF&}} with {{\t(0,1000,\frz360)}}animation{{\t(1000,2000,\frz0)}}"
                )
            }
            ComplexityLevel::Extreme => {
                format!(
                    r"{{\pos(100,200)\move(100,200,500,400)\fad(300,300)\t(0,500,\fscx120\fscy120)\t(500,1000,\fscx100\fscy100)\b1\i1\u1\s1\bord2\shad2\c&H00FF00&\3c&H0000FF&\4c&H000000&\alpha&H00\3a&H80}}{base_text}{{\b0\i0\u0\s0\r}} {{\k50}}with {{\k30}}karaoke {{\k40}}timing {{\k60}}and {{\k45}}complex {{\k35}}animations"
                )
            }
            ComplexityLevel::AnimeRealistic => {
                Self::generate_anime_dialogue(event_index, &base_text)
            }
            ComplexityLevel::MovieRealistic => {
                Self::generate_movie_dialogue(event_index, &base_text)
            }
            ComplexityLevel::KaraokeRealistic => {
                Self::generate_karaoke_dialogue(event_index, &base_text)
            }
            ComplexityLevel::SignRealistic => Self::generate_sign_dialogue(event_index, &base_text),
            ComplexityLevel::EducationalRealistic => {
                Self::generate_educational_dialogue(event_index, &base_text)
            }
        }
    }

    /// Generate anime-style dialogue with heavy effects
    fn generate_anime_dialogue(event_index: usize, base_text: &str) -> String {
        let patterns = [
            // Character speaking with glow effect
            format!(
                r"{{\an8\pos(960,80)\fad(250,250)\bord3\shad0\c&H00FFFFFF&\3c&H00FF8C00&}}{base_text}"
            ),
            // Thought bubble with transparency
            format!(
                r"{{\an5\pos(960,540)\fad(500,500)\alpha&H80&\bord2\c&H00E6E6FA&\3c&H00483D8B&}}{base_text}"
            ),
            // Dramatic effect with color change
            format!(
                r"{{\an2\pos(960,980)\fad(300,800)\b1\bord4\shad3\c&H0000FFFF&\3c&H000000FF&\4c&H00000000&\t(0,2000,\c&H00FF0000&)}}{base_text}"
            ),
            // Side character with positioning
            format!(
                r"{{\an7\pos(200,400)\fad(200,200)\bord2\c&H00FFFFFF&\3c&H00800080&}}{base_text}"
            ),
        ];
        patterns[event_index % patterns.len()].clone()
    }

    /// Generate movie-style dialogue (simple and clean)
    fn generate_movie_dialogue(event_index: usize, base_text: &str) -> String {
        let patterns = [
            // Standard dialogue
            base_text.to_string(),
            // Italic for emphasis
            format!(r"{{\i1}}{base_text}{{\i0}}"),
            // Bold for shouting
            format!(r"{{\b1}}{base_text}{{\b0}}"),
            // Different speaker position
            format!(r"{{\an8}}{base_text}"),
        ];
        patterns[event_index % patterns.len()].clone()
    }

    /// Generate karaoke-style dialogue with timing
    fn generate_karaoke_dialogue(event_index: usize, base_text: &str) -> String {
        let words: Vec<&str> = base_text.split_whitespace().collect();
        let mut karaoke_text = String::new();

        // Add base styling
        karaoke_text.push_str(
            r"{\an5\pos(960,540)\fad(200,200)\b1\bord2\shad1\c&H00FFFFFF&\3c&H00FF6347&}",
        );

        // Add karaoke timing for each word
        for (i, word) in words.iter().enumerate() {
            let timing = 50 + (i * 30); // Varying timing
            write!(karaoke_text, r"{{\k{timing}}}{word} ").unwrap();
        }

        // Add final effect
        if event_index % 3 == 0 {
            karaoke_text.push_str(r"{\t(2000,3000,\fscx120\fscy120\alpha&HFF&)}");
        }

        karaoke_text
    }

    /// Generate sign translation dialogue with positioning
    fn generate_sign_dialogue(event_index: usize, base_text: &str) -> String {
        let positions = [
            // Top signs
            (r"{\an8\pos(960,100)", "RESTAURANT"),
            (r"{\an9\pos(1700,150)", "EXIT"),
            (r"{\an7\pos(220,120)", "HOTEL"),
            // Screen text
            (r"{\an5\pos(960,540)", "NEWS FLASH"),
            // Bottom signs
            (r"{\an2\pos(960,950)", "SUBWAY"),
            // Side signs
            (r"{\an4\pos(100,540)", "STORE"),
        ];

        let (pos_tag, sign_type) = &positions[event_index % positions.len()];
        let sign_text = if base_text.contains("number") {
            format!(
                "{sign_type}: {}",
                base_text.replace("dialogue line", "sign")
            )
        } else {
            (*sign_type).to_string()
        };

        format!(
            r"{pos_tag}\fad(500,500)\bord3\shad2\c&H00000000&\3c&H00FFFFFF&\fn{{Arial}}\fs36}}{sign_text}"
        )
    }

    /// Generate educational content dialogue
    fn generate_educational_dialogue(event_index: usize, base_text: &str) -> String {
        let patterns = [
            // Main content
            format!(
                r"{{\an2\pos(960,900)\fad(200,200)\bord1\c&H00FFFFFF&}}{base_text} - This explains the concept in detail with proper formatting."
            ),
            // Question format
            format!(
                r"{{\an8\pos(960,150)\fad(200,200)\b1\c&H0000FFFF&}}Question {}: {base_text}",
                event_index + 1
            ),
            // Answer format
            format!(r"{{\an7\pos(100,400)\fad(200,200)\i1\c&H0000FF00&}}Answer: {base_text}"),
            // Definition
            format!(
                r"{{\an5\pos(960,540)\fad(200,200)\bord2\c&H00FFFFFF&\3c&H000080FF&}}Definition: {base_text}"
            ),
            // Example
            format!(r"{{\an1\pos(100,900)\fad(200,200)\c&H00FFFF00&}}Example: {base_text}"),
            // Summary
            format!(r"{{\an9\pos(1700,100)\fad(200,200)\b1\c&H00FF8000&}}Summary: {base_text}"),
        ];
        patterns[event_index % patterns.len()].clone()
    }
}
