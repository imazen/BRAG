use archmage::prelude::*;

macro_rules! neon_shuffle_row {
    ($name:ident, $mask:expr, $tail:expr) => {
        #[rite]
        pub(super) fn $name(_t: NeonToken, row: &mut [u8]) {
            let mask = vld1q_u8(&$mask);
            let n = row.len();
            let mut i = 0;
            while i + 16 <= n {
                let arr: &[u8; 16] = row[i..i + 16].try_into().unwrap();
                let out: &mut [u8; 16] = (&mut row[i..i + 16]).try_into().unwrap();
                vst1q_u8(out, vqtbl1q_u8(vld1q_u8(arr), mask));
                i += 16;
            }
            for px in row[i..].chunks_exact_mut(4) {
                let t: [u8; 4] = $tail(px);
                px.copy_from_slice(&t);
            }
        }
    };
}

macro_rules! neon_shuffle_copy_row {
    ($name:ident, $mask:expr, $tail:expr) => {
        #[rite]
        pub(super) fn $name(_t: NeonToken, src: &[u8], dst: &mut [u8]) {
            let mask = vld1q_u8(&$mask);
            let n = src.len().min(dst.len());
            let mut i = 0;
            while i + 16 <= n {
                let s: &[u8; 16] = src[i..i + 16].try_into().unwrap();
                let d: &mut [u8; 16] = (&mut dst[i..i + 16]).try_into().unwrap();
                vst1q_u8(d, vqtbl1q_u8(vld1q_u8(s), mask));
                i += 16;
            }
            for (s, d) in src[i..].chunks_exact(4).zip(dst[i..].chunks_exact_mut(4)) {
                let t: [u8; 4] = $tail(s);
                d.copy_from_slice(&t);
            }
        }
    };
}

neon_shuffle_row!(
    rgba_to_brag_row_neon,
    [2u8, 0, 3, 1, 6, 4, 7, 5, 10, 8, 11, 9, 14, 12, 15, 13],
    |p: &[u8]| [p[2], p[0], p[3], p[1]]
);
neon_shuffle_row!(
    brag_to_rgba_row_neon,
    [1u8, 3, 0, 2, 5, 7, 4, 6, 9, 11, 8, 10, 13, 15, 12, 14],
    |p: &[u8]| [p[1], p[3], p[0], p[2]]
);
neon_shuffle_row!(
    bgra_to_brag_row_neon,
    [0u8, 2, 3, 1, 4, 6, 7, 5, 8, 10, 11, 9, 12, 14, 15, 13],
    |p: &[u8]| [p[0], p[2], p[3], p[1]]
);
neon_shuffle_row!(
    brag_to_bgra_row_neon,
    [0u8, 3, 1, 2, 4, 7, 5, 6, 8, 11, 9, 10, 12, 15, 13, 14],
    |p: &[u8]| [p[0], p[3], p[1], p[2]]
);

neon_shuffle_copy_row!(
    copy_rgba_to_brag_row_neon,
    [2u8, 0, 3, 1, 6, 4, 7, 5, 10, 8, 11, 9, 14, 12, 15, 13],
    |s: &[u8]| [s[2], s[0], s[3], s[1]]
);
neon_shuffle_copy_row!(
    copy_brag_to_rgba_row_neon,
    [1u8, 3, 0, 2, 5, 7, 4, 6, 9, 11, 8, 10, 13, 15, 12, 14],
    |s: &[u8]| [s[1], s[3], s[0], s[2]]
);
neon_shuffle_copy_row!(
    copy_bgra_to_brag_row_neon,
    [0u8, 2, 3, 1, 4, 6, 7, 5, 8, 10, 11, 9, 12, 14, 15, 13],
    |s: &[u8]| [s[0], s[2], s[3], s[1]]
);
neon_shuffle_copy_row!(
    copy_brag_to_bgra_row_neon,
    [0u8, 3, 1, 2, 4, 7, 5, 6, 8, 11, 9, 10, 12, 15, 13, 14],
    |s: &[u8]| [s[0], s[3], s[1], s[2]]
);

// Contiguous wrappers
#[arcane]
pub(super) fn rgba_to_brag_impl_neon(t: NeonToken, b: &mut [u8]) {
    rgba_to_brag_row_neon(t, b);
}
#[arcane]
pub(super) fn copy_rgba_to_brag_impl_neon(t: NeonToken, s: &[u8], d: &mut [u8]) {
    copy_rgba_to_brag_row_neon(t, s, d);
}
#[arcane]
pub(super) fn brag_to_rgba_impl_neon(t: NeonToken, b: &mut [u8]) {
    brag_to_rgba_row_neon(t, b);
}
#[arcane]
pub(super) fn copy_brag_to_rgba_impl_neon(t: NeonToken, s: &[u8], d: &mut [u8]) {
    copy_brag_to_rgba_row_neon(t, s, d);
}
#[arcane]
pub(super) fn bgra_to_brag_impl_neon(t: NeonToken, b: &mut [u8]) {
    bgra_to_brag_row_neon(t, b);
}
#[arcane]
pub(super) fn copy_bgra_to_brag_impl_neon(t: NeonToken, s: &[u8], d: &mut [u8]) {
    copy_bgra_to_brag_row_neon(t, s, d);
}
#[arcane]
pub(super) fn brag_to_bgra_impl_neon(t: NeonToken, b: &mut [u8]) {
    brag_to_bgra_row_neon(t, b);
}
#[arcane]
pub(super) fn copy_brag_to_bgra_impl_neon(t: NeonToken, s: &[u8], d: &mut [u8]) {
    copy_brag_to_bgra_row_neon(t, s, d);
}

// Strided wrappers
macro_rules! strided_neon {
    ($name:ident, $row_fn:ident) => {
        #[arcane]
        pub(super) fn $name(t: NeonToken, buf: &mut [u8], w: usize, h: usize, stride: usize) {
            for y in 0..h {
                $row_fn(t, &mut buf[y * stride..][..w * 4]);
            }
        }
    };
}
macro_rules! strided_copy_neon {
    ($name:ident, $row_fn:ident) => {
        #[arcane]
        pub(super) fn $name(
            t: NeonToken,
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

strided_neon!(rgba_to_brag_strided_impl_neon, rgba_to_brag_row_neon);
strided_copy_neon!(
    copy_rgba_to_brag_strided_impl_neon,
    copy_rgba_to_brag_row_neon
);
strided_neon!(brag_to_rgba_strided_impl_neon, brag_to_rgba_row_neon);
strided_copy_neon!(
    copy_brag_to_rgba_strided_impl_neon,
    copy_brag_to_rgba_row_neon
);
strided_neon!(bgra_to_brag_strided_impl_neon, bgra_to_brag_row_neon);
strided_copy_neon!(
    copy_bgra_to_brag_strided_impl_neon,
    copy_bgra_to_brag_row_neon
);
strided_neon!(brag_to_bgra_strided_impl_neon, brag_to_bgra_row_neon);
strided_copy_neon!(
    copy_brag_to_bgra_strided_impl_neon,
    copy_brag_to_bgra_row_neon
);
