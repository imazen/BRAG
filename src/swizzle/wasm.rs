use archmage::prelude::*;

macro_rules! wasm_shuffle_row {
    ($name:ident, $mask:expr, $tail:expr) => {
        #[rite]
        pub(super) fn $name(_t: Wasm128Token, row: &mut [u8]) {
            let mask = $mask;
            let n = row.len();
            let mut i = 0;
            while i + 16 <= n {
                let arr: &[u8; 16] = row[i..i + 16].try_into().unwrap();
                let out: &mut [u8; 16] = (&mut row[i..i + 16]).try_into().unwrap();
                v128_store(out, i8x16_swizzle(v128_load(arr), mask));
                i += 16;
            }
            for px in row[i..].chunks_exact_mut(4) {
                let t: [u8; 4] = $tail(px);
                px.copy_from_slice(&t);
            }
        }
    };
}

macro_rules! wasm_shuffle_copy_row {
    ($name:ident, $mask:expr, $tail:expr) => {
        #[rite]
        pub(super) fn $name(_t: Wasm128Token, src: &[u8], dst: &mut [u8]) {
            let mask = $mask;
            let n = src.len().min(dst.len());
            let mut i = 0;
            while i + 16 <= n {
                let s: &[u8; 16] = src[i..i + 16].try_into().unwrap();
                let d: &mut [u8; 16] = (&mut dst[i..i + 16]).try_into().unwrap();
                v128_store(d, i8x16_swizzle(v128_load(s), mask));
                i += 16;
            }
            for (s, d) in src[i..].chunks_exact(4).zip(dst[i..].chunks_exact_mut(4)) {
                let t: [u8; 4] = $tail(s);
                d.copy_from_slice(&t);
            }
        }
    };
}

wasm_shuffle_row!(
    rgba_to_brag_row_wasm128,
    i8x16(2, 0, 3, 1, 6, 4, 7, 5, 10, 8, 11, 9, 14, 12, 15, 13),
    |p: &[u8]| [p[2], p[0], p[3], p[1]]
);
wasm_shuffle_row!(
    brag_to_rgba_row_wasm128,
    i8x16(1, 3, 0, 2, 5, 7, 4, 6, 9, 11, 8, 10, 13, 15, 12, 14),
    |p: &[u8]| [p[1], p[3], p[0], p[2]]
);
wasm_shuffle_row!(
    bgra_to_brag_row_wasm128,
    i8x16(0, 2, 3, 1, 4, 6, 7, 5, 8, 10, 11, 9, 12, 14, 15, 13),
    |p: &[u8]| [p[0], p[2], p[3], p[1]]
);
wasm_shuffle_row!(
    brag_to_bgra_row_wasm128,
    i8x16(0, 3, 1, 2, 4, 7, 5, 6, 8, 11, 9, 10, 12, 15, 13, 14),
    |p: &[u8]| [p[0], p[3], p[1], p[2]]
);

wasm_shuffle_copy_row!(
    copy_rgba_to_brag_row_wasm128,
    i8x16(2, 0, 3, 1, 6, 4, 7, 5, 10, 8, 11, 9, 14, 12, 15, 13),
    |s: &[u8]| [s[2], s[0], s[3], s[1]]
);
wasm_shuffle_copy_row!(
    copy_brag_to_rgba_row_wasm128,
    i8x16(1, 3, 0, 2, 5, 7, 4, 6, 9, 11, 8, 10, 13, 15, 12, 14),
    |s: &[u8]| [s[1], s[3], s[0], s[2]]
);
wasm_shuffle_copy_row!(
    copy_bgra_to_brag_row_wasm128,
    i8x16(0, 2, 3, 1, 4, 6, 7, 5, 8, 10, 11, 9, 12, 14, 15, 13),
    |s: &[u8]| [s[0], s[2], s[3], s[1]]
);
wasm_shuffle_copy_row!(
    copy_brag_to_bgra_row_wasm128,
    i8x16(0, 3, 1, 2, 4, 7, 5, 6, 8, 11, 9, 10, 12, 15, 13, 14),
    |s: &[u8]| [s[0], s[3], s[1], s[2]]
);

// Contiguous wrappers
#[arcane]
pub(super) fn rgba_to_brag_impl_wasm128(t: Wasm128Token, b: &mut [u8]) {
    rgba_to_brag_row_wasm128(t, b);
}
#[arcane]
pub(super) fn copy_rgba_to_brag_impl_wasm128(t: Wasm128Token, s: &[u8], d: &mut [u8]) {
    copy_rgba_to_brag_row_wasm128(t, s, d);
}
#[arcane]
pub(super) fn brag_to_rgba_impl_wasm128(t: Wasm128Token, b: &mut [u8]) {
    brag_to_rgba_row_wasm128(t, b);
}
#[arcane]
pub(super) fn copy_brag_to_rgba_impl_wasm128(t: Wasm128Token, s: &[u8], d: &mut [u8]) {
    copy_brag_to_rgba_row_wasm128(t, s, d);
}
#[arcane]
pub(super) fn bgra_to_brag_impl_wasm128(t: Wasm128Token, b: &mut [u8]) {
    bgra_to_brag_row_wasm128(t, b);
}
#[arcane]
pub(super) fn copy_bgra_to_brag_impl_wasm128(t: Wasm128Token, s: &[u8], d: &mut [u8]) {
    copy_bgra_to_brag_row_wasm128(t, s, d);
}
#[arcane]
pub(super) fn brag_to_bgra_impl_wasm128(t: Wasm128Token, b: &mut [u8]) {
    brag_to_bgra_row_wasm128(t, b);
}
#[arcane]
pub(super) fn copy_brag_to_bgra_impl_wasm128(t: Wasm128Token, s: &[u8], d: &mut [u8]) {
    copy_brag_to_bgra_row_wasm128(t, s, d);
}

// Strided wrappers
macro_rules! strided_wasm {
    ($name:ident, $row_fn:ident) => {
        #[arcane]
        pub(super) fn $name(t: Wasm128Token, buf: &mut [u8], w: usize, h: usize, stride: usize) {
            for y in 0..h {
                $row_fn(t, &mut buf[y * stride..][..w * 4]);
            }
        }
    };
}
macro_rules! strided_copy_wasm {
    ($name:ident, $row_fn:ident) => {
        #[arcane]
        pub(super) fn $name(
            t: Wasm128Token,
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

strided_wasm!(rgba_to_brag_strided_impl_wasm128, rgba_to_brag_row_wasm128);
strided_copy_wasm!(
    copy_rgba_to_brag_strided_impl_wasm128,
    copy_rgba_to_brag_row_wasm128
);
strided_wasm!(brag_to_rgba_strided_impl_wasm128, brag_to_rgba_row_wasm128);
strided_copy_wasm!(
    copy_brag_to_rgba_strided_impl_wasm128,
    copy_brag_to_rgba_row_wasm128
);
strided_wasm!(bgra_to_brag_strided_impl_wasm128, bgra_to_brag_row_wasm128);
strided_copy_wasm!(
    copy_bgra_to_brag_strided_impl_wasm128,
    copy_bgra_to_brag_row_wasm128
);
strided_wasm!(brag_to_bgra_strided_impl_wasm128, brag_to_bgra_row_wasm128);
strided_copy_wasm!(
    copy_brag_to_bgra_strided_impl_wasm128,
    copy_brag_to_bgra_row_wasm128
);
