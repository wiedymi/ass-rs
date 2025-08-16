//! Animation processing for ASS effects

/// Calculate progress for \move animation
pub fn calculate_move_progress(time_cs: u32, t1: u32, t2: u32) -> f32 {
    if time_cs <= t1 {
        0.0
    } else if time_cs >= t2 {
        1.0
    } else {
        (time_cs - t1) as f32 / (t2 - t1) as f32
    }
}

/// Calculate progress for \fade animation
pub fn calculate_fade_progress(time_cs: u32, t1: u32, t2: u32) -> f32 {
    if time_cs <= t1 {
        0.0
    } else if time_cs >= t2 {
        1.0
    } else {
        (time_cs - t1) as f32 / (t2 - t1) as f32
    }
}
