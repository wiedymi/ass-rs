//! Font selection and coverage matching for shaping.

#[cfg(feature = "nostd")]
use alloc::{format, string::ToString, sync::Arc};
#[cfg(not(feature = "nostd"))]
use std::{string::ToString, sync::Arc};

use crate::utils::RenderError;
use fontdb::{Database as FontDatabase, ID as FontId};

/// Find matching font in database with fallback support for CJK/Chinese/Japanese/Korean text
fn find_font(
    font_database: &FontDatabase,
    family: &str,
    bold: bool,
    italic: bool,
) -> Result<FontId, RenderError> {
    // First try the requested font
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(family), fontdb::Family::SansSerif],
        weight: if bold {
            fontdb::Weight::BOLD
        } else {
            fontdb::Weight::NORMAL
        },
        stretch: fontdb::Stretch::Normal,
        style: if italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        },
    };

    if let Some(id) = font_database.query(&query) {
        return Ok(id);
    }

    // If the requested font contains "CJK" or is a known Japanese font name, try fallbacks
    let is_japanese_font = family.contains("CJK")
        || family.contains("Noto Sans JP")
        || family.contains("Hiragino")
        || family.contains("Yu Gothic")
        || family.contains("MS Gothic")
        || family.contains("Meiryo");

    if is_japanese_font {
        // Try common Japanese font fallbacks
        let japanese_fallbacks = [
            "Hiragino Sans",
            "Hiragino Kaku Gothic Pro",
            "Yu Gothic",
            "MS Gothic",
            "Meiryo",
            "Noto Sans CJK JP",
            "Noto Sans JP",
        ];

        for fallback in &japanese_fallbacks {
            let fallback_query = fontdb::Query {
                families: &[fontdb::Family::Name(fallback)],
                weight: if bold {
                    fontdb::Weight::BOLD
                } else {
                    fontdb::Weight::NORMAL
                },
                stretch: fontdb::Stretch::Normal,
                style: if italic {
                    fontdb::Style::Italic
                } else {
                    fontdb::Style::Normal
                },
            };

            if let Some(id) = font_database.query(&fallback_query) {
                return Ok(id);
            }
        }
    }

    // Final fallback to any available font
    let final_query = fontdb::Query {
        families: &[fontdb::Family::SansSerif, fontdb::Family::Serif],
        weight: if bold {
            fontdb::Weight::BOLD
        } else {
            fontdb::Weight::NORMAL
        },
        stretch: fontdb::Stretch::Normal,
        style: if italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        },
    };

    if let Some(id) = font_database.query(&final_query) {
        return Ok(id);
    }

    // Last resort: any face actually loaded in the database. fontdb resolves the
    // generic SansSerif/Serif families to platform default names (e.g. "Arial")
    // that may not exist on the host, so fall back to whatever font is present.
    font_database
        .faces()
        .next()
        .map(|face| face.id)
        .ok_or_else(|| {
            RenderError::FontError(format!(
                "Font '{family}' not found and no fallback available"
            ))
        })
}

/// Determine if text contains significant CJK/Hangul content
fn text_has_cjk_or_hangul(text: &str) -> bool {
    text.chars().any(|ch| {
        let u = ch as u32;
        // CJK Unified Ideographs
        (0x4E00..=0x9FFF).contains(&u)
            // CJK Unified Ideographs Extension A/B (basic check)
            || (0x3400..=0x4DBF).contains(&u)
            || (0x20000..=0x2A6DF).contains(&u)
            // Hangul Syllables + Jamo
            || (0xAC00..=0xD7AF).contains(&u)
            || (0x1100..=0x11FF).contains(&u)
            // Hiragana, Katakana
            || (0x3040..=0x30FF).contains(&u)
    })
}

/// Check how many characters of `text` are supported by the font face
fn font_support_count(
    font_database: &FontDatabase,
    font_id: FontId,
    text: &str,
) -> Result<usize, RenderError> {
    let (source, index) = font_database
        .face_source(font_id)
        .ok_or_else(|| RenderError::FontError("Failed to load font data".to_string()))?;

    let data: Arc<dyn AsRef<[u8]> + Send + Sync> = match source {
        fontdb::Source::Binary(data) => data,
        fontdb::Source::File(path) => {
            #[cfg(not(feature = "nostd"))]
            {
                std::sync::Arc::new(std::fs::read(&path).map_err(|e| {
                    RenderError::FontError(format!("Failed to read font file: {e}"))
                })?)
            }
            #[cfg(feature = "nostd")]
            {
                return Err(RenderError::FontError(
                    "File reading not supported in no_std mode".into(),
                ));
            }
        }
        fontdb::Source::SharedFile(_, data) => data,
    };

    let face = ttf_parser::Face::parse(data.as_ref().as_ref(), index)
        .map_err(|_| RenderError::FontError("Failed to parse font".to_string()))?;

    // Count support for BMP chars in text
    let mut count = 0usize;
    for ch in text.chars() {
        if face.glyph_index(ch).is_some() {
            count += 1;
        }
    }
    Ok(count)
}

/// Attempt to pick a font that best covers the input text while respecting the requested family if possible
pub fn find_font_for_text(
    font_database: &FontDatabase,
    family: &str,
    bold: bool,
    italic: bool,
    text: &str,
) -> Result<FontId, RenderError> {
    // First, try the requested font (or its immediate fallback in find_font)
    let primary = find_font(font_database, family, bold, italic)?;
    let primary_support = font_support_count(font_database, primary, text)?;
    let text_len = text.chars().count().max(1);

    if primary_support == text_len {
        return Ok(primary);
    }

    // If the text contains CJK/Hangul, try curated fallback lists first
    if text_has_cjk_or_hangul(text) {
        let curated_lists: &[&[&str]] = &[
            // Simplified Chinese
            &[
                "Noto Sans CJK SC",
                "Noto Sans SC",
                "Source Han Sans CN",
                "PingFang SC",
                "Microsoft YaHei",
                "SimHei",
                "SimSun",
            ],
            // Traditional Chinese
            &[
                "Noto Sans CJK TC",
                "Noto Sans TC",
                "Source Han Sans TW",
                "PingFang TC",
                "Heiti TC",
            ],
            // Japanese
            &[
                "Noto Sans CJK JP",
                "Noto Sans JP",
                "Hiragino Sans",
                "Hiragino Kaku Gothic Pro",
                "Yu Gothic",
                "MS Gothic",
                "Meiryo",
            ],
            // Korean
            &[
                "Noto Sans CJK KR",
                "Noto Sans KR",
                "Apple SD Gothic Neo",
                "Malgun Gothic",
            ],
            // Generic CJK
            &["Source Han Sans", "Noto Sans CJK"],
        ];

        let mut best_font = None;
        let mut best_support = primary_support;

        for list in curated_lists {
            for &name in *list {
                let query = fontdb::Query {
                    families: &[fontdb::Family::Name(name)],
                    weight: if bold {
                        fontdb::Weight::BOLD
                    } else {
                        fontdb::Weight::NORMAL
                    },
                    stretch: fontdb::Stretch::Normal,
                    style: if italic {
                        fontdb::Style::Italic
                    } else {
                        fontdb::Style::Normal
                    },
                };
                if let Some(id) = font_database.query(&query) {
                    let support = font_support_count(font_database, id, text)?;
                    if support == text_len {
                        return Ok(id);
                    }
                    if support > best_support {
                        best_support = support;
                        best_font = Some(id);
                    }
                }
            }
        }

        if let Some(id) = best_font {
            return Ok(id);
        }
    }

    // As a last resort, scan all faces to find the best coverage
    let mut best_font = primary;
    let mut best_support = primary_support;
    for face in font_database.faces() {
        // Skip if it's the same font
        if face.id == primary {
            continue;
        }
        let id = face.id;
        let support = font_support_count(font_database, id, text)?;
        if support > best_support {
            best_support = support;
            best_font = id;
            if best_support == text_len {
                break;
            }
        }
    }

    Ok(best_font)
}
