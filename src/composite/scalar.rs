use archmage::prelude::*;

// ============================================================
// div255 — exact integer division by 255
//
// For all x in 0..=65025 (the range of u8 * u8):
//   div255(x) == round(x / 255.0)
// ============================================================

#[inline(always)]
fn div255(x: u32) -> u8 {
    let t = x + 128;
    ((t + (t >> 8)) >> 8) as u8
}

// ============================================================
// Premultiply / Unpremultiply — #[autoversion]
//
// LLVM auto-vectorizes these well (proven by garb's u8 premul).
// archmage generates per-tier dispatch; LLVM sees target features
// and vectorizes accordingly.
// ============================================================

/// BRAG layout: B₀ R₁ A₂ G₃ — alpha at byte offset 2.
#[autoversion(v3, neon, wasm128)]
pub(super) fn premul_brag_impl(buf: &mut [u8]) {
    for px in buf.chunks_exact_mut(4) {
        let a = px[2] as u32; // BRAG: alpha at index 2
        px[0] = div255(px[0] as u32 * a); // B
        px[1] = div255(px[1] as u32 * a); // R
        // px[2] = alpha (unchanged)
        px[3] = div255(px[3] as u32 * a); // G
    }
}

#[autoversion(v3, neon, wasm128)]
pub(super) fn unpremul_brag_impl(buf: &mut [u8]) {
    for px in buf.chunks_exact_mut(4) {
        let a = px[2]; // BRAG: alpha at index 2
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[3] = 0;
        } else if a < 255 {
            let a16 = a as u16;
            px[0] = ((px[0] as u16 * 255 + a16 / 2) / a16).min(255) as u8;
            px[1] = ((px[1] as u16 * 255 + a16 / 2) / a16).min(255) as u8;
            px[3] = ((px[3] as u16 * 255 + a16 / 2) / a16).min(255) as u8;
        }
        // a == 255: no change needed
    }
}

// ============================================================
// SrcOver — scalar row implementations for incant! dispatch
//
// BRAG alpha at byte 2.
// Formula (premultiplied): dst' = src + dst * (255 - src_a) / 255
// ============================================================

pub(super) fn src_over_brag_row_scalar(_token: ScalarToken, src: &[u8], dst: &mut [u8]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        let src_a = s[2]; // BRAG alpha
        if src_a == 255 {
            d[0] = s[0];
            d[1] = s[1];
            d[2] = s[2];
            d[3] = s[3];
        } else if src_a > 0 {
            let inv_a = (255 - src_a) as u32;
            d[0] = s[0].wrapping_add(div255(d[0] as u32 * inv_a));
            d[1] = s[1].wrapping_add(div255(d[1] as u32 * inv_a));
            d[2] = s[2].wrapping_add(div255(d[2] as u32 * inv_a));
            d[3] = s[3].wrapping_add(div255(d[3] as u32 * inv_a));
        }
        // src_a == 0: dst unchanged
    }
}

pub(super) fn src_over_solid_brag_row_scalar(_token: ScalarToken, dst: &mut [u8], color: &[u8; 4]) {
    let src_a = color[2]; // BRAG alpha
    if src_a == 255 {
        for d in dst.chunks_exact_mut(4) {
            d[0] = color[0];
            d[1] = color[1];
            d[2] = color[2];
            d[3] = color[3];
        }
    } else if src_a > 0 {
        let inv_a = (255 - src_a) as u32;
        for d in dst.chunks_exact_mut(4) {
            d[0] = color[0].wrapping_add(div255(d[0] as u32 * inv_a));
            d[1] = color[1].wrapping_add(div255(d[1] as u32 * inv_a));
            d[2] = color[2].wrapping_add(div255(d[2] as u32 * inv_a));
            d[3] = color[3].wrapping_add(div255(d[3] as u32 * inv_a));
        }
    }
    // src_a == 0: dst unchanged
}

// ============================================================
// Scalar wrappers for incant! dispatch
// ============================================================

pub(super) fn src_over_brag_impl_scalar(t: ScalarToken, src: &[u8], dst: &mut [u8]) {
    src_over_brag_row_scalar(t, src, dst);
}

pub(super) fn src_over_solid_brag_impl_scalar(t: ScalarToken, dst: &mut [u8], color: &[u8; 4]) {
    src_over_solid_brag_row_scalar(t, dst, color);
}

// ============================================================
// f32 variants — autoversioned (LLVM vectorizes FMA well)
//
// BRAG f32 layout: [B, R, A, G] per pixel, alpha at index 2.
// ============================================================

#[autoversion(v3, neon, wasm128)]
pub(super) fn src_over_brag_f32_impl(src: &[f32], dst: &mut [f32]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        let inv_a = 1.0 - s[2]; // BRAG alpha at index 2
        d[0] = s[0] + d[0] * inv_a;
        d[1] = s[1] + d[1] * inv_a;
        d[2] = s[2] + d[2] * inv_a;
        d[3] = s[3] + d[3] * inv_a;
    }
}

#[autoversion(v3, neon, wasm128)]
pub(super) fn premul_brag_f32_impl(buf: &mut [f32]) {
    for px in buf.chunks_exact_mut(4) {
        let a = px[2]; // BRAG alpha at index 2
        px[0] *= a;
        px[1] *= a;
        // px[2] = alpha unchanged
        px[3] *= a;
    }
}
