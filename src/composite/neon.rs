use archmage::prelude::*;

// BRAG alpha broadcast: byte 2 of each 4-byte pixel → all 4 positions.
const BRAG_ALPHA_TBL: [u8; 16] = [2, 2, 2, 2, 6, 6, 6, 6, 10, 10, 10, 10, 14, 14, 14, 14];

#[rite]
pub(super) fn src_over_brag_row_neon(_token: NeonToken, src: &[u8], dst: &mut [u8]) {
    let tbl = vld1q_u8(&BRAG_ALPHA_TBL);
    let bias = vdupq_n_u16(128);

    let n = src.len().min(dst.len());
    let mut i = 0;

    while i + 16 <= n {
        let s: &[u8; 16] = src[i..i + 16].try_into().unwrap();
        let src_v = vld1q_u8(s);
        let d: &[u8; 16] = dst[i..i + 16].try_into().unwrap();
        let dst_v = vld1q_u8(d);

        // Broadcast alpha, compute 255 - alpha via bitwise NOT
        let src_alpha = vqtbl1q_u8(src_v, tbl);
        let inv_alpha = vmvnq_u8(src_alpha);

        // Low half: widening multiply + div255
        let dst_lo = vmull_u8(vget_low_u8(dst_v), vget_low_u8(inv_alpha));
        let t_lo = vaddq_u16(dst_lo, bias);
        let r_lo = vshrq_n_u16::<8>(vaddq_u16(t_lo, vshrq_n_u16::<8>(t_lo)));

        // High half
        let dst_hi = vmull_u8(vget_high_u8(dst_v), vget_high_u8(inv_alpha));
        let t_hi = vaddq_u16(dst_hi, bias);
        let r_hi = vshrq_n_u16::<8>(vaddq_u16(t_hi, vshrq_n_u16::<8>(t_hi)));

        // Narrow + add src
        let blended = vcombine_u8(vqmovn_u16(r_lo), vqmovn_u16(r_hi));
        let result = vaddq_u8(src_v, blended);

        let out: &mut [u8; 16] = (&mut dst[i..i + 16]).try_into().unwrap();
        vst1q_u8(out, result);
        i += 16;
    }

    // Scalar tail
    scalar_src_over_tail(&src[i..], &mut dst[i..]);
}

#[rite]
pub(super) fn src_over_solid_brag_row_neon(_token: NeonToken, dst: &mut [u8], color: &[u8; 4]) {
    let color_arr: [u8; 16] = [
        color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3], color[0],
        color[1], color[2], color[3], color[0], color[1], color[2], color[3],
    ];
    let src_v = vld1q_u8(&color_arr);
    let inv_a = vdupq_n_u8(255u8.wrapping_sub(color[2]));
    let bias = vdupq_n_u16(128);

    let n = dst.len();
    let mut i = 0;

    while i + 16 <= n {
        let d: &[u8; 16] = dst[i..i + 16].try_into().unwrap();
        let dst_v = vld1q_u8(d);

        let dst_lo = vmull_u8(vget_low_u8(dst_v), vget_low_u8(inv_a));
        let t_lo = vaddq_u16(dst_lo, bias);
        let r_lo = vshrq_n_u16::<8>(vaddq_u16(t_lo, vshrq_n_u16::<8>(t_lo)));

        let dst_hi = vmull_u8(vget_high_u8(dst_v), vget_high_u8(inv_a));
        let t_hi = vaddq_u16(dst_hi, bias);
        let r_hi = vshrq_n_u16::<8>(vaddq_u16(t_hi, vshrq_n_u16::<8>(t_hi)));

        let blended = vcombine_u8(vqmovn_u16(r_lo), vqmovn_u16(r_hi));
        let result = vaddq_u8(src_v, blended);

        let out: &mut [u8; 16] = (&mut dst[i..i + 16]).try_into().unwrap();
        vst1q_u8(out, result);
        i += 16;
    }

    scalar_src_over_solid_tail(&mut dst[i..], color);
}

// Scalar tails

#[inline(always)]
fn div255(x: u32) -> u8 {
    let t = x + 128;
    ((t + (t >> 8)) >> 8) as u8
}

fn scalar_src_over_tail(src: &[u8], dst: &mut [u8]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        let inv_a = (255 - s[2]) as u32;
        d[0] = s[0].wrapping_add(div255(d[0] as u32 * inv_a));
        d[1] = s[1].wrapping_add(div255(d[1] as u32 * inv_a));
        d[2] = s[2].wrapping_add(div255(d[2] as u32 * inv_a));
        d[3] = s[3].wrapping_add(div255(d[3] as u32 * inv_a));
    }
}

fn scalar_src_over_solid_tail(dst: &mut [u8], color: &[u8; 4]) {
    let inv_a = (255 - color[2]) as u32;
    for d in dst.chunks_exact_mut(4) {
        d[0] = color[0].wrapping_add(div255(d[0] as u32 * inv_a));
        d[1] = color[1].wrapping_add(div255(d[1] as u32 * inv_a));
        d[2] = color[2].wrapping_add(div255(d[2] as u32 * inv_a));
        d[3] = color[3].wrapping_add(div255(d[3] as u32 * inv_a));
    }
}

// arcane wrappers

#[arcane]
pub(super) fn src_over_brag_impl_neon(t: NeonToken, src: &[u8], dst: &mut [u8]) {
    src_over_brag_row_neon(t, src, dst);
}

#[arcane]
pub(super) fn src_over_solid_brag_impl_neon(t: NeonToken, dst: &mut [u8], color: &[u8; 4]) {
    src_over_solid_brag_row_neon(t, dst, color);
}
