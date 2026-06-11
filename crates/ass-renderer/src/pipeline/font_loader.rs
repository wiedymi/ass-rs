//! Font loading from ASS scripts including embedded fonts

use ass_core::parser::Script;
use fontdb::Database as FontDatabase;

/// Lazily-loaded, process-wide system font database, shared via `Arc`.
///
/// `load_system_fonts` scans every installed font and is expensive (tens of ms),
/// so it is run once and the result shared. Read-only callers — notably the
/// software backend, which builds a fresh instance every frame — clone the `Arc`
/// instead of re-scanning, which was previously the dominant per-frame cost.
#[cfg(not(feature = "nostd"))]
pub fn shared_system_fonts() -> std::sync::Arc<FontDatabase> {
    use std::sync::{Arc, OnceLock};
    static CACHE: OnceLock<Arc<FontDatabase>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut db = FontDatabase::new();
            db.load_system_fonts();
            // Best-effort fallback dirs for broader (e.g. CJK) coverage on Unix;
            // Windows fonts are already covered by load_system_fonts.
            let dirs = [
                "/usr/share/fonts",
                "/usr/local/share/fonts",
                "/System/Library/Fonts",
                "/System/Library/Fonts/Supplemental",
                "/Library/Fonts",
            ];
            for dir in dirs {
                if std::path::Path::new(dir).exists() {
                    db.load_fonts_dir(dir);
                }
            }
            if let Ok(home) = std::env::var("HOME") {
                let user_fonts = format!("{home}/.fonts");
                if std::path::Path::new(&user_fonts).exists() {
                    db.load_fonts_dir(&user_fonts);
                }
            }
            Arc::new(db)
        })
        .clone()
}

/// Load embedded fonts from ASS script into font database
pub fn load_embedded_fonts(script: &Script, font_database: &mut FontDatabase) {
    // Check if the script has a Fonts section
    if let Some(fonts) = script.sections().iter().find_map(|section| {
        if let ass_core::parser::ast::Section::Fonts(fonts) = section {
            Some(fonts)
        } else {
            None
        }
    }) {
        // Process each embedded font
        for font in fonts {
            // Decode the UU-encoded font data
            match font.decode_data() {
                Ok(font_data) => {
                    // Load the font into the database
                    // fontdb's load_font_data takes ownership of the data
                    font_database.load_font_data(font_data);
                }
                Err(_e) => {}
            }
        }
    }
}

/// Load fonts from file paths specified in script metadata
pub fn load_font_files(
    script: &Script,
    #[cfg_attr(feature = "nostd", allow(unused_variables))] font_database: &mut FontDatabase,
) {
    // Check Script Info section for font file references
    if let Some(script_info) = script.sections().iter().find_map(|section| {
        if let ass_core::parser::ast::Section::ScriptInfo(info) = section {
            Some(info)
        } else {
            None
        }
    }) {
        // Look for font file declarations in fields
        for (key, value) in &script_info.fields {
            // Some scripts use "Font:" or "Fontname:" fields to specify external fonts
            if key.to_lowercase().contains("font") {
                let _font_path = value.trim();

                #[cfg(not(feature = "nostd"))]
                {
                    // Try to load as a file path if it exists
                    let path = std::path::Path::new(_font_path);
                    if path.exists() && path.is_file() {
                        let _ = font_database.load_font_file(path);
                    }
                }
            }
        }
    }
}

/// Load all fonts referenced in an ASS script
pub fn load_script_fonts(script: &Script, font_database: &mut FontDatabase) {
    // Load embedded fonts from [Fonts] section
    load_embedded_fonts(script, font_database);

    // Load external font files if referenced
    load_font_files(script, font_database);
}
