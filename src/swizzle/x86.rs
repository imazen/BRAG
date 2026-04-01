use archmage::prelude::*;

const RGBA_TO_BRAG: [i8; 32] = [
    2, 0, 3, 1, 6, 4, 7, 5, 10, 8, 11, 9, 14, 12, 15, 13, 2, 0, 3, 1, 6, 4, 7, 5, 10, 8, 11, 9, 14,
    12, 15, 13,
];
const BRAG_TO_RGBA: [i8; 32] = [
    1, 3, 0, 2, 5, 7, 4, 6, 9, 11, 8, 10, 13, 15, 12, 14, 1, 3, 0, 2, 5, 7, 4, 6, 9, 11, 8, 10, 13,
    15, 12, 14,
];
const BGRA_TO_BRAG: [i8; 32] = [
    0, 2, 3, 1, 4, 6, 7, 5, 8, 10, 11, 9, 12, 14, 15, 13, 0, 2, 3, 1, 4, 6, 7, 5, 8, 10, 11, 9, 12,
    14, 15, 13,
];
const BRAG_TO_BGRA: [i8; 32] = [
    0, 3, 1, 2, 4, 7, 5, 6, 8, 11, 9, 10, 12, 15, 13, 14, 0, 3, 1, 2, 4, 7, 5, 6, 8, 11, 9, 10, 12,
    15, 13, 14,
];

// Row function helper: AVX2 shuffle + scalar tail for inplace.
// Split load/store to satisfy borrow checker (same pattern as garb).
macro_rules! avx2_row {
    ($name:ident, $mask:ident, [$i0:literal,$i1:literal,$i2:literal,$i3:literal]) => {
        #[rite]
        pub(super) fn $name(_t: X64V3Token, row: &mut [u8]) {
            let mask = _mm256_loadu_si256(&$mask);
            let n = row.len();
            let mut i = 0;
            while i + 32 <= n {
                let arr: &[u8; 32] = row[i..i + 32].try_into().unwrap();
                let v = _mm256_loadu_si256(arr);
                let shuffled = _mm256_shuffle_epi8(v, mask);
                let out: &mut [u8; 32] = (&mut row[i..i + 32]).try_into().unwrap();
                _mm256_storeu_si256(out, shuffled);
                i += 32;
            }
            for px in row[i..].chunks_exact_mut(4) {
                let tmp = [px[$i0], px[$i1], px[$i2], px[$i3]];
                px.copy_from_slice(&tmp);
            }
        }
    };
}

macro_rules! avx2_copy_row {
    ($name:ident, $mask:ident, [$i0:literal,$i1:literal,$i2:literal,$i3:literal]) => {
        #[rite]
        pub(super) fn $name(_t: X64V3Token, src: &[u8], dst: &mut [u8]) {
            let mask = _mm256_loadu_si256(&$mask);
            let n = src.len().min(dst.len());
            let mut i = 0;
            while i + 32 <= n {
                let s: &[u8; 32] = src[i..i + 32].try_into().unwrap();
                let v = _mm256_shuffle_epi8(_mm256_loadu_si256(s), mask);
                let d: &mut [u8; 32] = (&mut dst[i..i + 32]).try_into().unwrap();
                _mm256_storeu_si256(d, v);
                i += 32;
            }
            for (s, d) in src[i..].chunks_exact(4).zip(dst[i..].chunks_exact_mut(4)) {
                d[0] = s[$i0];
                d[1] = s[$i1];
                d[2] = s[$i2];
                d[3] = s[$i3];
            }
        }
    };
}

// Row functions
avx2_row!(rgba_to_brag_row_v3, RGBA_TO_BRAG, [2, 0, 3, 1]);
avx2_row!(brag_to_rgba_row_v3, BRAG_TO_RGBA, [1, 3, 0, 2]);
avx2_row!(bgra_to_brag_row_v3, BGRA_TO_BRAG, [0, 2, 3, 1]);
avx2_row!(brag_to_bgra_row_v3, BRAG_TO_BGRA, [0, 3, 1, 2]);

avx2_copy_row!(copy_rgba_to_brag_row_v3, RGBA_TO_BRAG, [2, 0, 3, 1]);
avx2_copy_row!(copy_brag_to_rgba_row_v3, BRAG_TO_RGBA, [1, 3, 0, 2]);
avx2_copy_row!(copy_bgra_to_brag_row_v3, BGRA_TO_BRAG, [0, 2, 3, 1]);
avx2_copy_row!(copy_brag_to_bgra_row_v3, BRAG_TO_BGRA, [0, 3, 1, 2]);

// Contiguous wrappers
#[arcane]
pub(super) fn rgba_to_brag_impl_v3(t: X64V3Token, b: &mut [u8]) {
    rgba_to_brag_row_v3(t, b);
}
#[arcane]
pub(super) fn copy_rgba_to_brag_impl_v3(t: X64V3Token, s: &[u8], d: &mut [u8]) {
    copy_rgba_to_brag_row_v3(t, s, d);
}
#[arcane]
pub(super) fn brag_to_rgba_impl_v3(t: X64V3Token, b: &mut [u8]) {
    brag_to_rgba_row_v3(t, b);
}
#[arcane]
pub(super) fn copy_brag_to_rgba_impl_v3(t: X64V3Token, s: &[u8], d: &mut [u8]) {
    copy_brag_to_rgba_row_v3(t, s, d);
}
#[arcane]
pub(super) fn bgra_to_brag_impl_v3(t: X64V3Token, b: &mut [u8]) {
    bgra_to_brag_row_v3(t, b);
}
#[arcane]
pub(super) fn copy_bgra_to_brag_impl_v3(t: X64V3Token, s: &[u8], d: &mut [u8]) {
    copy_bgra_to_brag_row_v3(t, s, d);
}
#[arcane]
pub(super) fn brag_to_bgra_impl_v3(t: X64V3Token, b: &mut [u8]) {
    brag_to_bgra_row_v3(t, b);
}
#[arcane]
pub(super) fn copy_brag_to_bgra_impl_v3(t: X64V3Token, s: &[u8], d: &mut [u8]) {
    copy_brag_to_bgra_row_v3(t, s, d);
}

// Strided wrappers
macro_rules! strided_v3 {
    ($name:ident, $row_fn:ident) => {
        #[arcane]
        pub(super) fn $name(t: X64V3Token, buf: &mut [u8], w: usize, h: usize, stride: usize) {
            for y in 0..h {
                $row_fn(t, &mut buf[y * stride..][..w * 4]);
            }
        }
    };
}
macro_rules! strided_copy_v3 {
    ($name:ident, $row_fn:ident) => {
        #[arcane]
        pub(super) fn $name(
            t: X64V3Token,
            src: &[u8],
            dst: &mut [u8],
            w: usize,
            h: usize,
            ss: usize,
            ds: usize,
        ) {
            for y in 0..h {
                $row_fn(t, &src[y * ss..][..w * 4], &mut dst[y * ds..][..w * 4]);
            }
        }
    };
}

strided_v3!(rgba_to_brag_strided_impl_v3, rgba_to_brag_row_v3);
strided_copy_v3!(copy_rgba_to_brag_strided_impl_v3, copy_rgba_to_brag_row_v3);
strided_v3!(brag_to_rgba_strided_impl_v3, brag_to_rgba_row_v3);
strided_copy_v3!(copy_brag_to_rgba_strided_impl_v3, copy_brag_to_rgba_row_v3);
strided_v3!(bgra_to_brag_strided_impl_v3, bgra_to_brag_row_v3);
strided_copy_v3!(copy_bgra_to_brag_strided_impl_v3, copy_bgra_to_brag_row_v3);
strided_v3!(brag_to_bgra_strided_impl_v3, brag_to_bgra_row_v3);
strided_copy_v3!(copy_brag_to_bgra_strided_impl_v3, copy_brag_to_bgra_row_v3);
