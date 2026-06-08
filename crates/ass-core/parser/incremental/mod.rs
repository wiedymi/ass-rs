//! Incremental parsing utilities for efficient text updates

mod line_calc;
mod range_adjust;
mod section_bounds;

#[cfg(test)]
mod tests;

pub use line_calc::{calculate_line_number, calculate_line_range};
pub use range_adjust::{adjust_range_for_change, TextChange};
pub use section_bounds::{find_section_end, find_section_header_start};
