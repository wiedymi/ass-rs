/// Handle PlayResX/PlayResY coordinate scaling
pub fn scale_coordinates(
    x: f32,
    y: f32,
    play_res_x: f32,
    play_res_y: f32,
    screen_width: f32,
    screen_height: f32,
) -> (f32, f32) {
    let scale_x = screen_width / play_res_x;
    let scale_y = screen_height / play_res_y;
    (x * scale_x, y * scale_y)
}

/// Convert SSA legacy alignment to ASS alignment
pub fn convert_ssa_alignment(ssa_align: u8) -> u8 {
    // SSA: 1=left, 2=center, 3=right
    // +0=sub, +4=title, +8=top, +128=mid-title
    let h_align = ssa_align & 3;
    let v_align = (ssa_align >> 2) & 3;

    let h_part = match h_align {
        1 => 1, // Left
        2 => 2, // Center
        3 => 3, // Right
        _ => 2, // Default center
    };

    let v_part = match v_align {
        0 => 0,     // Bottom
        1 | 3 => 3, // Middle (title or mid-title)
        2 => 6,     // Top
        _ => 0,     // Default bottom
    };

    h_part + v_part
}
