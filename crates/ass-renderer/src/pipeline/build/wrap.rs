//! Word-wrapping of tagged text segments for the software pipeline.

#[cfg(feature = "nostd")]
use alloc::{string::String, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

use crate::pipeline::{shaping::shape_text_cached, text_segmenter::TextSegment};

impl super::SoftwarePipeline {
    /// Word-wrap a logical line (one or more tagged segments) to fit `max_width`
    /// (render pixels), preserving per-segment tags and balancing line widths to
    /// approximate libass smart wrapping (WrapStyle 0). Width is measured with the
    /// line's leading font — exact for the common single-font / mid-line
    /// colour-change case; mixed font SIZES within one line are approximated.
    /// Returns one entry per wrapped line (a line that already fits is one entry).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn wrap_segments(
        &self,
        line: &[TextSegment],
        default_font_name: &str,
        default_font_size: f32,
        default_scale_y: f32,
        default_bold: bool,
        default_italic: bool,
        default_spacing: f32,
        default_scale_x: f32,
        scale_y: f32,
        max_width: f32,
        balance: bool,
    ) -> Vec<Vec<TextSegment>> {
        let Some(lead) = line.first() else {
            return vec![line.to_vec()];
        };

        // Flatten to characters with a per-character owning-segment index.
        let mut chars: Vec<char> = Vec::new();
        let mut owner: Vec<usize> = Vec::new();
        for (idx, seg) in line.iter().enumerate() {
            for ch in seg.text.chars() {
                chars.push(ch);
                owner.push(idx);
            }
        }
        let full: String = chars.iter().collect();
        let mut words: Vec<&str> = full.split(' ').collect();
        // Drop empty tokens from a trailing space (e.g. "...land, \N"): libass trims
        // trailing whitespace before measuring, so it must not count toward the wrap
        // width or it can force a spurious extra break. The space stays in the
        // rendered (invisible) line tail via the char-range rebuild below.
        while words.len() > 1 && words.last().is_some_and(|w| w.is_empty()) {
            words.pop();
        }
        if words.len() <= 1 {
            return vec![line.to_vec()];
        }

        // Measure with the leading segment's resolved render font, reproducing the
        // render's geometry exactly so a line wraps iff it would actually overflow:
        // glyph advances at the rendered size, plus letter spacing applied BETWEEN
        // glyphs (libass counts N-1 gaps; the trailing glyph has no spacing), with
        // the whole run scaled by `\fscx` to mirror the render's post-transform.
        // Without the spacing term a line measures narrower than it draws and fails
        // to wrap when libass does.
        let font = lead.tags.font.name.as_deref().unwrap_or(default_font_name);
        let size = lead.tags.font.size.unwrap_or(default_font_size)
            * scale_y
            * (lead.tags.font.scale_y.unwrap_or(default_scale_y) / 100.0)
            * self.dpi_scale;
        let bold = lead.tags.formatting.bold.unwrap_or(default_bold);
        let italic = lead.tags.formatting.italic.unwrap_or(default_italic);
        // `\fsp`/style spacing is a script-space gap; the advances here are at the
        // screen `size`, so scale spacing to screen too (libass uses fsp * scale).
        // Adding it unscaled over-measures the line and forces an early wrap (an
        // extra line vs libass on borderline-width subtitles).
        let spacing = lead.tags.font.spacing.unwrap_or(default_spacing) * scale_y;
        // The shaped advance already carries \fscy (folded into `size`), and the
        // render's net horizontal scale is \fscx, so the wrap multiplier is the
        // x/y ratio (reduces to \fscx/100 in the common \fscy=100 case).
        let scale_y_pct = lead.tags.font.scale_y.unwrap_or(default_scale_y);
        let sx = if scale_y_pct.abs() > 0.01 {
            lead.tags.font.scale_x.unwrap_or(default_scale_x) / scale_y_pct
        } else {
            lead.tags.font.scale_x.unwrap_or(default_scale_x) / 100.0
        };
        let measure = |s: &str| -> (f32, f32, f32) {
            // (advance, ink_min, ink_max)
            shape_text_cached(s, font, size, bold, italic, &self.font_database)
                .map_or((0.0, 0.0, 0.0), |sh| (sh.width, sh.ink_min, sh.ink_max))
        };
        let word_m: Vec<(f32, f32, f32)> = words.iter().map(|w| measure(w)).collect();
        let word_adv: Vec<f32> = word_m.iter().map(|m| m.0).collect();
        // Per-word side bearings: lead = ink left edge, trail = advance - ink right.
        // libass measures a line on ink width (x_max - x_min), which is the advance
        // box minus the first word's lead and the last word's trail.
        let word_lead: Vec<f32> = word_m.iter().map(|m| m.1).collect();
        let word_trail: Vec<f32> = word_m.iter().map(|m| (m.0 - m.2).max(0.0)).collect();
        let word_glyphs: Vec<usize> = words.iter().map(|w| w.chars().count()).collect();
        // Advance of one space glyph, isolated from neighbouring side bearings.
        let space_adv = (measure("x x").0 - measure("xx").0).max(0.0);

        // Ink width of a contiguous run of words [start..=end]: advances plus the
        // inter-glyph spacing, scaled by `\fscx`, minus the run's leading/trailing
        // side bearings (so it matches libass's ink-extent wrap threshold).
        let line_ink = |adv: f32, glyphs: usize, start: usize, end: usize| -> f32 {
            let advance_box = adv + spacing * glyphs.saturating_sub(1) as f32;
            sx * (advance_box - word_lead[start] - word_trail[end])
        };

        // Greedily pack words under `limit`, returning word indices that start a line.
        let fill = |limit: f32| -> Vec<usize> {
            let mut starts: Vec<usize> = Vec::new();
            let mut line_start = 0usize;
            let mut cur_adv = 0.0;
            let mut cur_glyphs = 0usize;
            let mut started = false;
            for i in 0..words.len() {
                let (add_adv, add_glyphs) = if started {
                    (space_adv + word_adv[i], 1 + word_glyphs[i])
                } else {
                    (word_adv[i], word_glyphs[i])
                };
                if started
                    && line_ink(cur_adv + add_adv, cur_glyphs + add_glyphs, line_start, i) > limit
                {
                    starts.push(i);
                    line_start = i;
                    cur_adv = word_adv[i];
                    cur_glyphs = word_glyphs[i];
                } else {
                    cur_adv += add_adv;
                    cur_glyphs += add_glyphs;
                    started = true;
                }
            }
            starts
        };
        let base = fill(max_width);
        if base.is_empty() {
            return vec![line.to_vec()];
        }

        // WrapStyle 0/3 (`balance`): find the smallest width limit that still yields
        // the greedy break count, biasing earlier lines wider like libass smart
        // wrapping. WrapStyle 1 keeps the raw greedy (end-of-line) breaks.
        let line_starts: Vec<usize> = if balance {
            let target = base.len();
            let max_word = (0..words.len())
                .map(|i| line_ink(word_adv[i], word_glyphs[i], i, i))
                .fold(0.0_f32, f32::max);
            let (mut lo, mut hi) = (max_word, max_width);
            for _ in 0..24 {
                let mid = (lo + hi) / 2.0;
                if fill(mid).len() <= target {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            core::iter::once(0).chain(fill(hi)).collect()
        } else {
            core::iter::once(0).chain(base).collect()
        };

        // Character offset where each word begins (words are space-separated).
        let mut word_char_start = Vec::with_capacity(words.len());
        let mut pos = 0usize;
        for (i, word) in words.iter().enumerate() {
            word_char_start.push(pos);
            pos += word.chars().count();
            if i + 1 < words.len() {
                pos += 1; // the separating space
            }
        }

        // Slice the flattened line into per-wrap char ranges and rebuild segments.
        let mut out: Vec<Vec<TextSegment>> = Vec::with_capacity(line_starts.len());
        for (k, &start_word) in line_starts.iter().enumerate() {
            let start_char = word_char_start[start_word];
            let end_char = if k + 1 < line_starts.len() {
                word_char_start[line_starts[k + 1]] - 1 // drop the break space
            } else {
                chars.len()
            };
            let mut segs: Vec<TextSegment> = Vec::new();
            let mut i = start_char;
            while i < end_char {
                let seg_idx = owner[i];
                let mut text = String::new();
                while i < end_char && owner[i] == seg_idx {
                    text.push(chars[i]);
                    i += 1;
                }
                segs.push(TextSegment {
                    text,
                    start: line[seg_idx].start,
                    end: line[seg_idx].end,
                    tags: line[seg_idx].tags.clone(),
                });
            }
            if !segs.is_empty() {
                out.push(segs);
            }
        }
        out
    }
}
