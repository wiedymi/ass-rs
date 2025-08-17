//! Font loading from ASS scripts including embedded fonts

use ass_core::parser::Script;
use fontdb::Database as FontDatabase;

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
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        eprintln!("Found {} embedded fonts in script", fonts.len());

        // Process each embedded font
        for font in fonts {
            // Decode the UU-encoded font data
            match font.decode_data() {
                Ok(font_data) => {
                    // Load the font into the database
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!(
                        "Loading embedded font: {} ({} bytes)",
                        font.filename,
                        font_data.len()
                    );

                    // fontdb's load_font_data takes ownership of the data
                    font_database.load_font_data(font_data);

                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("Successfully loaded embedded font: {}", font.filename);
                }
                Err(_e) => {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("Failed to decode embedded font {}: {:?}", font.filename, _e);
                }
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
                        #[cfg(all(debug_assertions, not(feature = "nostd")))]
                        eprintln!("Loading external font file: {_font_path}");

                        if let Err(_e) = font_database.load_font_file(path) {
                            #[cfg(all(debug_assertions, not(feature = "nostd")))]
                            eprintln!("Failed to load font file {_font_path}: {_e:?}");
                        }
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
