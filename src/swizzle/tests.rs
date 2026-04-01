extern crate alloc;
extern crate std;

use super::*;
use alloc::{vec, vec::Vec};
use archmage::testing::{CompileTimePolicy, for_each_token_permutation};

fn policy() -> CompileTimePolicy {
    if std::env::var_os("CI").is_some() {
        CompileTimePolicy::Fail
    } else {
        CompileTimePolicy::WarnStderr
    }
}

fn make_4bpp(n: usize) -> Vec<u8> {
    (0..n * 4).map(|i| (i % 251) as u8).collect()
}

const TEST_SIZES: &[usize] = &[1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 31, 32, 33, 64, 100];

// ── Reference implementations ──────────────────────────────────────

fn ref_rgba_to_brag(src: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    for px in out.chunks_exact_mut(4) {
        let [r, g, b, a] = [px[0], px[1], px[2], px[3]];
        px[0] = b;
        px[1] = r;
        px[2] = a;
        px[3] = g;
    }
    out
}
fn ref_brag_to_rgba(src: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    for px in out.chunks_exact_mut(4) {
        let [b, r, a, g] = [px[0], px[1], px[2], px[3]];
        px[0] = r;
        px[1] = g;
        px[2] = b;
        px[3] = a;
    }
    out
}
fn ref_bgra_to_brag(src: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    for px in out.chunks_exact_mut(4) {
        let [b, g, r, a] = [px[0], px[1], px[2], px[3]];
        px[0] = b;
        px[1] = r;
        px[2] = a;
        px[3] = g;
    }
    out
}
fn ref_brag_to_bgra(src: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    for px in out.chunks_exact_mut(4) {
        let [b, r, a, g] = [px[0], px[1], px[2], px[3]];
        px[0] = b;
        px[1] = g;
        px[2] = r;
        px[3] = a;
    }
    out
}

// ── Permutation tests (exercises all SIMD tiers) ───────────────────

macro_rules! perm_test_inplace {
    ($name:ident, $fn:ident, $ref_fn:ident) => {
        #[test]
        fn $name() {
            let report = for_each_token_permutation(policy(), |perm| {
                for &n in TEST_SIZES {
                    let mut data = make_4bpp(n);
                    let expected = $ref_fn(&data);
                    $fn(&mut data).unwrap();
                    assert_eq!(data, expected, "{} n={n} tier={perm}", stringify!($fn));
                }
            });
            std::eprintln!("{}: {report}", stringify!($name));
        }
    };
}

macro_rules! perm_test_copy {
    ($name:ident, $fn:ident, $ref_fn:ident) => {
        #[test]
        fn $name() {
            let report = for_each_token_permutation(policy(), |perm| {
                for &n in TEST_SIZES {
                    let src = make_4bpp(n);
                    let expected = $ref_fn(&src);
                    let mut dst = vec![0u8; n * 4];
                    $fn(&src, &mut dst).unwrap();
                    assert_eq!(dst, expected, "{} n={n} tier={perm}", stringify!($fn));
                }
            });
            std::eprintln!("{}: {report}", stringify!($name));
        }
    };
}

perm_test_inplace!(
    perm_rgba_to_brag_inplace,
    rgba_to_brag_inplace,
    ref_rgba_to_brag
);
perm_test_copy!(perm_rgba_to_brag_copy, rgba_to_brag, ref_rgba_to_brag);
perm_test_inplace!(
    perm_brag_to_rgba_inplace,
    brag_to_rgba_inplace,
    ref_brag_to_rgba
);
perm_test_copy!(perm_brag_to_rgba_copy, brag_to_rgba, ref_brag_to_rgba);
perm_test_inplace!(
    perm_bgra_to_brag_inplace,
    bgra_to_brag_inplace,
    ref_bgra_to_brag
);
perm_test_copy!(perm_bgra_to_brag_copy, bgra_to_brag, ref_bgra_to_brag);
perm_test_inplace!(
    perm_brag_to_bgra_inplace,
    brag_to_bgra_inplace,
    ref_brag_to_bgra
);
perm_test_copy!(perm_brag_to_bgra_copy, brag_to_bgra, ref_brag_to_bgra);

// ── Round-trip tests ───────────────────────────────────────────────

#[test]
fn round_trip_rgba() {
    for &n in TEST_SIZES {
        let original = make_4bpp(n);
        let mut buf = original.clone();
        rgba_to_brag_inplace(&mut buf).unwrap();
        brag_to_rgba_inplace(&mut buf).unwrap();
        assert_eq!(buf, original, "RGBA→BRAG→RGBA round-trip n={n}");
    }
}

#[test]
fn round_trip_bgra() {
    for &n in TEST_SIZES {
        let original = make_4bpp(n);
        let mut buf = original.clone();
        bgra_to_brag_inplace(&mut buf).unwrap();
        brag_to_bgra_inplace(&mut buf).unwrap();
        assert_eq!(buf, original, "BGRA→BRAG→BGRA round-trip n={n}");
    }
}

// ── Strided tests ──────────────────────────────────────────────────

#[test]
fn strided_rgba_to_brag() {
    let w = 4usize;
    let h = 3usize;
    let stride = w * 4 + 8; // 8 bytes padding per row
    let mut buf = vec![0xFFu8; stride * h];
    // Fill pixel data only (not padding)
    for y in 0..h {
        for x in 0..w {
            let i = y * stride + x * 4;
            buf[i] = (y * w + x) as u8; // R
            buf[i + 1] = 0; // G
            buf[i + 2] = 255; // B
            buf[i + 3] = 128; // A
        }
    }
    rgba_to_brag_inplace_strided(&mut buf, w, h, stride).unwrap();
    // Check first pixel: was [R,G,B,A] = [0,0,255,128], now [B,R,A,G] = [255,0,128,0]
    assert_eq!(buf[0], 255); // B
    assert_eq!(buf[1], 0); // R
    assert_eq!(buf[2], 128); // A
    assert_eq!(buf[3], 0); // G
    // Padding bytes should be untouched
    assert_eq!(buf[w * 4], 0xFF);
}

#[test]
fn strided_copy_bgra_to_brag() {
    let w = 8usize;
    let h = 2usize;
    let src_stride = w * 4 + 4;
    let dst_stride = w * 4;
    let src = vec![42u8; src_stride * h];
    let mut dst = vec![0u8; dst_stride * h];
    bgra_to_brag_strided(&src, &mut dst, w, h, src_stride, dst_stride).unwrap();
    // All source bytes were 42, so after shuffle all dst bytes should be 42
    for &b in &dst {
        assert_eq!(b, 42);
    }
}

// ── Error tests ────────────────────────────────────────────────────

#[test]
fn error_not_aligned() {
    assert_eq!(
        rgba_to_brag_inplace(&mut [0; 3]),
        Err(SwizzleError::NotPixelAligned)
    );
    assert_eq!(
        brag_to_rgba(&[0; 5], &mut [0; 8]),
        Err(SwizzleError::NotPixelAligned)
    );
}

#[test]
fn error_length_mismatch() {
    assert_eq!(
        rgba_to_brag(&[0; 8], &mut [0; 4]),
        Err(SwizzleError::LengthMismatch)
    );
}

#[test]
fn error_invalid_stride() {
    // Stride too small
    assert_eq!(
        rgba_to_brag_inplace_strided(&mut [0; 64], 4, 2, 8),
        Err(SwizzleError::InvalidStride)
    );
}
