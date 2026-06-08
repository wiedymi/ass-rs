//! Section boundary discovery for incremental re-parsing

use crate::parser::errors::ParseError;
use crate::parser::SectionType;

/// Find the start of a section header before the given position
///
/// # Errors
///
/// Returns [`ParseError::SectionNotFound`] if the section header cannot be found
pub fn find_section_header_start(
    source: &str,
    start_hint: usize,
    section_type: SectionType,
) -> Result<usize, ParseError> {
    // Expected header for each section type
    let header = match section_type {
        SectionType::ScriptInfo => "[Script Info]",
        SectionType::Styles => "[V4+ Styles]",
        SectionType::Events => "[Events]",
        SectionType::Fonts => "[Fonts]",
        SectionType::Graphics => "[Graphics]",
    };

    // Search backwards from start_hint for the header
    let search_start = start_hint.saturating_sub(header.len() + 100); // Look back up to 100 chars
    let search_text = &source[search_start..start_hint.min(source.len())];

    search_text
        .rfind(header)
        .map_or(Err(ParseError::SectionNotFound), |pos| {
            // Found the header, now find the start of the line
            let header_pos = search_start + pos;
            let line_start = source[..header_pos].rfind('\n').map_or(0, |p| p + 1);
            Ok(line_start)
        })
}

/// Find the end of a section (start of next section or end of file)
///
/// # Errors
///
/// Returns [`ParseError`] if an error occurs while finding the section end
pub fn find_section_end(
    source: &str,
    end_hint: usize,
    _section_type: SectionType,
) -> Result<usize, ParseError> {
    // Look for the next section header
    let section_headers = [
        "[Script Info]",
        "[V4+ Styles]",
        "[Events]",
        "[Fonts]",
        "[Graphics]",
    ];

    let search_text = &source[end_hint..];

    // Find the nearest section header
    let mut min_pos = None;
    for header in &section_headers {
        if let Some(pos) = search_text.find(header) {
            min_pos = Some(min_pos.map_or(pos, |min: usize| min.min(pos)));
        }
    }

    min_pos.map_or(Ok(source.len()), |pos| {
        // Found next section, return start of that line
        let next_section_pos = end_hint + pos;
        let line_start = source[..next_section_pos].rfind('\n').map_or(0, |p| p + 1);
        Ok(line_start)
    })
}
