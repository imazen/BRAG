use archmage::prelude::*;

#[rite]
pub(super) fn src_over_brag_row_wasm128(_token: Wasm128Token, src: &[u8], dst: &mut [u8]) {
    // BRAG alpha broadcast: byte 2 of each 4-byte pixel → all 4 positions
    let alpha_shuf = i8x16(2, 2, 2, 2, 6, 6, 6, 6, 10, 10, 10, 10, 14, 14, 14, 14);
    let all_ff = i8x16_splat(-1); // 0xFF
    let bias = i16x8_splat(128);

    let n = src.len().min(dst.len());
    let mut i = 0;

    while i + 16 <= n {
        let s: &[u8; 16] = src[i..i + 16].try_into().unwrap();
        let src_v = v128_load(s);
        let d: &[u8; 16] = dst[i..i + 16].try_into().unwrap();
        let dst_v = v128_load(d);

        // Broadcast alpha, XOR with 0xFF = bitwise NOT = 255 - x
        let src_alpha = i8x16_swizzle(src_v, alpha_shuf);
        let inv_alpha = v128_xor(src_alpha, all_ff);

        // Low half: widen to u16, multiply, div255
        let dst_lo = i16x8_extend_low_u8x16(dst_v);
        let inv_lo = i16x8_extend_low_u8x16(inv_alpha);
        let prod_lo = i16x8_mul(dst_lo, inv_lo);
        let t_lo = i16x8_add(prod_lo, bias);
        let r_lo = u16x8_shr(i16x8_add(t_lo, u16x8_shr(t_lo, 8)), 8);

        // High half
        let dst_hi = i16x8_extend_high_u8x16(dst_v);
        let inv_hi = i16x8_extend_high_u8x16(inv_alpha);
        let prod_hi = i16x8_mul(dst_hi, inv_hi);
        let t_hi = i16x8_add(prod_hi, bias);
        let r_hi = u16x8_shr(i16x8_add(t_hi, u16x8_shr(t_hi, 8)), 8);

        // Narrow u16→u8 (saturating), add src
        let blended = u8x16_narrow_i16x8(r_lo, r_hi);
        let result = i8x16_add(src_v, blended);

        let out: &mut [u8; 16] = (&mut dst[i..i + 16]).try_into().unwrap();
        v128_store(out, result);
        i += 16;
    }

    scalar_src_over_tail(&src[i..], &mut dst[i..]);
}

#[rite]
pub(super) fn src_over_solid_brag_row_wasm128(
    _token: Wasm128Token,
    dst: &mut [u8],
    color: &[u8; 4],
) {
    let color_arr: [u8; 16] = [
        color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3], color[0],
        color[1], color[2], color[3], color[0], color[1], color[2], color[3],
    ];
    let src_v = v128_load(&color_arr);
    let inv_a_byte = 255u8.wrapping_sub(color[2]);
    let inv_alpha = u8x16_splat(inv_a_byte);
    let bias = i16x8_splat(128);

    let n = dst.len();
    let mut i = 0;

    while i + 16 <= n {
        let d: &[u8; 16] = dst[i..i + 16].try_into().unwrap();
        let dst_v = v128_load(d);

        let dst_lo = i16x8_extend_low_u8x16(dst_v);
        let inv_lo = i16x8_extend_low_u8x16(inv_alpha);
        let prod_lo = i16x8_mul(dst_lo, inv_lo);
        let t_lo = i16x8_add(prod_lo, bias);
        let r_lo = u16x8_shr(i16x8_add(t_lo, u16x8_shr(t_lo, 8)), 8);

        let dst_hi = i16x8_extend_high_u8x16(dst_v);
        let inv_hi = i16x8_extend_high_u8x16(inv_alpha);
        let prod_hi = i16x8_mul(dst_hi, inv_hi);
        let t_hi = i16x8_add(prod_hi, bias);
        let r_hi = u16x8_shr(i16x8_add(t_hi, u16x8_shr(t_hi, 8)), 8);

        let blended = u8x16_narrow_i16x8(r_lo, r_hi);
        let result = i8x16_add(src_v, blended);

        let out: &mut [u8; 16] = (&mut dst[i..i + 16]).try_into().unwrap();
        v128_store(out, result);
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
pub(super) fn src_over_brag_impl_wasm128(t: Wasm128Token, src: &[u8], dst: &mut [u8]) {
    src_over_brag_row_wasm128(t, src, dst);
}

#[arcane]
pub(super) fn src_over_solid_brag_impl_wasm128(t: Wasm128Token, dst: &mut [u8], color: &[u8; 4]) {
    src_over_solid_brag_row_wasm128(t, dst, color);
}
