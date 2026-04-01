use archmage::prelude::*;

// ============================================================
// BRAG alpha broadcast: replicate byte 2 of each 4-byte pixel
// to all 4 positions in that pixel.
//
// BRAG layout per pixel: [B₀, R₁, A₂, G₃]
// 4 pixels in 128-bit lane: alpha at bytes 2, 6, 10, 14
// ============================================================

const BRAG_ALPHA_BROADCAST: [i8; 32] = [
    2, 2, 2, 2, 6, 6, 6, 6, 10, 10, 10, 10, 14, 14, 14, 14, 2, 2, 2, 2, 6, 6, 6, 6, 10, 10, 10, 10,
    14, 14, 14, 14,
];

// ============================================================
// SrcOver — AVX2: 8 pixels (32 bytes) per iteration
//
// dst' = src + dst × (255 − src_alpha) / 255
//
// Pipeline:
//   1. shuffle to broadcast alpha
//   2. sub to get inv_alpha
//   3. unpack u8→u16 (lo + hi halves)
//   4. mullo_epi16 for dst × inv_alpha
//   5. div255 in u16: t = prod + 128; (t + (t >> 8)) >> 8
//   6. packus_epi16 to narrow back to u8
//   7. add_epi8 for src + result (no saturation: premul guarantees ≤255)
// ============================================================

#[rite]
pub(super) fn src_over_brag_row_v3(_token: X64V3Token, src: &[u8], dst: &mut [u8]) {
    let alpha_shuf = _mm256_loadu_si256(&BRAG_ALPHA_BROADCAST);
    let all_ff = _mm256_set1_epi8(-1i8); // 0xFF
    let bias = _mm256_set1_epi16(128);
    let zero = _mm256_setzero_si256();

    let n = src.len().min(dst.len());
    let mut i = 0;

    while i + 32 <= n {
        let s: &[u8; 32] = src[i..i + 32].try_into().unwrap();
        let src_v = _mm256_loadu_si256(s);
        let d: &[u8; 32] = dst[i..i + 32].try_into().unwrap();
        let dst_v = _mm256_loadu_si256(d);

        // Broadcast each pixel's alpha (byte 2) to all 4 bytes
        let src_alpha = _mm256_shuffle_epi8(src_v, alpha_shuf);
        // inv_alpha = 255 - alpha
        let inv_alpha = _mm256_sub_epi8(all_ff, src_alpha);

        // Low half: widen to u16, multiply, div255
        let dst_lo = _mm256_unpacklo_epi8(dst_v, zero);
        let inv_lo = _mm256_unpacklo_epi8(inv_alpha, zero);
        let prod_lo = _mm256_mullo_epi16(dst_lo, inv_lo);
        let t_lo = _mm256_add_epi16(prod_lo, bias);
        let r_lo = _mm256_srli_epi16::<8>(_mm256_add_epi16(t_lo, _mm256_srli_epi16::<8>(t_lo)));

        // High half: same
        let dst_hi = _mm256_unpackhi_epi8(dst_v, zero);
        let inv_hi = _mm256_unpackhi_epi8(inv_alpha, zero);
        let prod_hi = _mm256_mullo_epi16(dst_hi, inv_hi);
        let t_hi = _mm256_add_epi16(prod_hi, bias);
        let r_hi = _mm256_srli_epi16::<8>(_mm256_add_epi16(t_hi, _mm256_srli_epi16::<8>(t_hi)));

        // Pack u16→u8 and add source
        let blended = _mm256_packus_epi16(r_lo, r_hi);
        let result = _mm256_add_epi8(src_v, blended);

        let out: &mut [u8; 32] = (&mut dst[i..i + 32]).try_into().unwrap();
        _mm256_storeu_si256(out, result);
        i += 32;
    }

    // Scalar tail (0–7 pixels)
    scalar_src_over_tail(&src[i..], &mut dst[i..]);
}

#[rite]
pub(super) fn src_over_solid_brag_row_v3(_token: X64V3Token, dst: &mut [u8], color: &[u8; 4]) {
    // Broadcast color to all 8 pixel positions
    let color_128: [u8; 16] = [
        color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3], color[0],
        color[1], color[2], color[3], color[0], color[1], color[2], color[3],
    ];
    let src_v = _mm256_broadcastsi128_si256(_mm_loadu_si128(&color_128));

    // inv_alpha is uniform
    let inv_a = 255u8.wrapping_sub(color[2]);
    let inv_alpha = _mm256_set1_epi8(inv_a as i8);
    let bias = _mm256_set1_epi16(128);
    let zero = _mm256_setzero_si256();

    let n = dst.len();
    let mut i = 0;

    while i + 32 <= n {
        let d: &[u8; 32] = dst[i..i + 32].try_into().unwrap();
        let dst_v = _mm256_loadu_si256(d);

        let dst_lo = _mm256_unpacklo_epi8(dst_v, zero);
        let inv_lo = _mm256_unpacklo_epi8(inv_alpha, zero);
        let prod_lo = _mm256_mullo_epi16(dst_lo, inv_lo);
        let t_lo = _mm256_add_epi16(prod_lo, bias);
        let r_lo = _mm256_srli_epi16::<8>(_mm256_add_epi16(t_lo, _mm256_srli_epi16::<8>(t_lo)));

        let dst_hi = _mm256_unpackhi_epi8(dst_v, zero);
        let inv_hi = _mm256_unpackhi_epi8(inv_alpha, zero);
        let prod_hi = _mm256_mullo_epi16(dst_hi, inv_hi);
        let t_hi = _mm256_add_epi16(prod_hi, bias);
        let r_hi = _mm256_srli_epi16::<8>(_mm256_add_epi16(t_hi, _mm256_srli_epi16::<8>(t_hi)));

        let blended = _mm256_packus_epi16(r_lo, r_hi);
        let result = _mm256_add_epi8(src_v, blended);

        let out: &mut [u8; 32] = (&mut dst[i..i + 32]).try_into().unwrap();
        _mm256_storeu_si256(out, result);
        i += 32;
    }

    // Scalar tail
    scalar_src_over_solid_tail(&mut dst[i..], color);
}

// ============================================================
// Scalar tails (shared with all SIMD paths)
// ============================================================

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

// ============================================================
// arcane wrappers for incant! dispatch
// ============================================================

#[arcane]
pub(super) fn src_over_brag_impl_v3(t: X64V3Token, src: &[u8], dst: &mut [u8]) {
    src_over_brag_row_v3(t, src, dst);
}

#[arcane]
pub(super) fn src_over_solid_brag_impl_v3(t: X64V3Token, dst: &mut [u8], color: &[u8; 4]) {
    src_over_solid_brag_row_v3(t, dst, color);
}
