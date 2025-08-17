//! Layer compositing module

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use crate::pipeline::IntermediateLayer;
use crate::utils::RenderError;

/// Compositing mode
#[derive(Debug, Clone, Copy)]
pub enum CompositeMode {
    /// Normal alpha blending
    Normal,
    /// Additive blending
    Add,
    /// Multiply blending
    Multiply,
    /// XOR blending for shapes
    Xor,
}

/// Composite multiple layers into a single buffer
pub fn composite_layers(
    layers: &[IntermediateLayer],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, RenderError> {
    let mut output = vec![0u8; (width * height * 4) as usize];

    for layer in layers {
        composite_layer(layer, &mut output, width, height)?;
    }

    Ok(output)
}

/// Composite a single layer onto the output buffer
fn composite_layer(
    layer: &IntermediateLayer,
    output: &mut [u8],
    width: u32,
    height: u32,
) -> Result<(), RenderError> {
    // TODO: Implement proper compositing
    let _ = (layer, output, width, height);
    Ok(())
}

/// Apply alpha blending using libass-compatible formula
///
/// Uses integer arithmetic with specific rounding to match libass exactly:
/// - Multiply and add 255 for rounding bias
/// - Right shift by 8 instead of division by 255
/// - This matches libass's MUL_BITMAPS/IMUL_BITMAPS operations
pub fn alpha_blend(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    // Use integer arithmetic matching libass implementation
    let src_alpha = src[3] as u32;

    if src_alpha == 0 {
        // Fully transparent source, keep destination
        return dst;
    }

    if src_alpha == 255 && dst[3] == 0 {
        // Opaque source over transparent destination
        return src;
    }

    // libass uses this formula: ((src * alpha + 255) >> 8)
    // The +255 provides rounding bias to match libass exactly
    let inv_src_alpha = 255 - src_alpha;

    // Blend colors using libass formula with rounding
    let out_r = ((src[0] as u32 * src_alpha + dst[0] as u32 * inv_src_alpha + 255) >> 8) as u8;
    let out_g = ((src[1] as u32 * src_alpha + dst[1] as u32 * inv_src_alpha + 255) >> 8) as u8;
    let out_b = ((src[2] as u32 * src_alpha + dst[2] as u32 * inv_src_alpha + 255) >> 8) as u8;

    // Alpha calculation using same integer formula
    let dst_alpha = dst[3] as u32;
    let out_alpha = ((src_alpha * 255 + dst_alpha * inv_src_alpha + 255) >> 8) as u8;

    [out_r, out_g, out_b, out_alpha]
}
