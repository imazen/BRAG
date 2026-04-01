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

// ── Reference implementations ──────────────────────────────────────

fn ref_div255(x: u32) -> u8 {
    let t = x + 128;
    ((t + (t >> 8)) >> 8) as u8
}

fn ref_premul(buf: &[u8]) -> Vec<u8> {
    let mut out = buf.to_vec();
    for px in out.chunks_exact_mut(4) {
        let a = px[2] as u32; // BRAG alpha at byte 2
        px[0] = ref_div255(px[0] as u32 * a);
        px[1] = ref_div255(px[1] as u32 * a);
        px[3] = ref_div255(px[3] as u32 * a);
    }
    out
}

fn ref_unpremul(buf: &[u8]) -> Vec<u8> {
    let mut out = buf.to_vec();
    for px in out.chunks_exact_mut(4) {
        let a = px[2];
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
    }
    out
}

fn ref_src_over(src: &[u8], dst: &[u8]) -> Vec<u8> {
    let mut out = dst.to_vec();
    for (s, d) in src.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
        let inv_a = (255 - s[2]) as u32;
        d[0] = s[0].wrapping_add(ref_div255(d[0] as u32 * inv_a));
        d[1] = s[1].wrapping_add(ref_div255(d[1] as u32 * inv_a));
        d[2] = s[2].wrapping_add(ref_div255(d[2] as u32 * inv_a));
        d[3] = s[3].wrapping_add(ref_div255(d[3] as u32 * inv_a));
    }
    out
}

// ── Test data generators ───────────────────────────────────────────

fn make_brag(n_pixels: usize) -> Vec<u8> {
    (0..n_pixels * 4).map(|i| (i % 251) as u8).collect()
}

fn make_premul_brag(n_pixels: usize) -> Vec<u8> {
    // Generate valid premultiplied pixels (each channel <= alpha)
    let mut buf = Vec::with_capacity(n_pixels * 4);
    for i in 0..n_pixels {
        let a = ((i * 37 + 13) % 256) as u8;
        let b = ((i * 53 + 7) % (a as usize + 1)) as u8;
        let r = ((i * 41 + 3) % (a as usize + 1)) as u8;
        let g = ((i * 67 + 11) % (a as usize + 1)) as u8;
        buf.extend_from_slice(&[b, r, a, g]); // BRAG order
    }
    buf
}

const TEST_PIXEL_COUNTS: &[usize] = &[1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 31, 32, 33, 64, 100];

// ── div255 exhaustive ──────────────────────────────────────────────

#[test]
fn div255_exhaustive() {
    for a in 0u32..=255 {
        for c in 0u32..=255 {
            let x = c * a;
            let expected = ((x as f64 / 255.0) + 0.5) as u8;
            let got = ref_div255(x);
            assert_eq!(
                got, expected,
                "div255({x}) = {got}, expected {expected} (c={c}, a={a})"
            );
        }
    }
}

// ── Premultiply ────────────────────────────────────────────────────

#[test]
fn premultiply_known() {
    // Opaque pixel: unchanged
    let mut buf = [64u8, 255, 255, 128]; // B=64, R=255, A=255, G=128
    premultiply(&mut buf).unwrap();
    assert_eq!(buf, [64, 255, 255, 128]);

    // Transparent pixel: zeroed
    let mut buf = [64u8, 255, 0, 128];
    premultiply(&mut buf).unwrap();
    assert_eq!(buf, [0, 0, 0, 0]);

    // Half alpha
    let mut buf = [200u8, 100, 128, 50]; // B=200, R=100, A=128, G=50
    premultiply(&mut buf).unwrap();
    assert_eq!(buf[2], 128); // alpha unchanged
    // 200 * 128 / 255 ≈ 100
    assert!(buf[0].abs_diff(100) <= 1);
}

#[test]
fn premultiply_sizes() {
    for &n in TEST_PIXEL_COUNTS {
        let mut data = make_brag(n);
        let expected = ref_premul(&data);
        premultiply(&mut data).unwrap();
        assert_eq!(data, expected, "premultiply n={n}");
    }
}

// ── Unpremultiply ──────────────────────────────────────────────────

#[test]
fn unpremultiply_known() {
    // Transparent: stays zero
    let mut buf = [0u8, 0, 0, 0];
    unpremultiply(&mut buf).unwrap();
    assert_eq!(buf, [0, 0, 0, 0]);

    // Opaque: unchanged
    let mut buf = [64u8, 255, 255, 128];
    unpremultiply(&mut buf).unwrap();
    assert_eq!(buf, [64, 255, 255, 128]);
}

#[test]
fn premul_unpremul_round_trip() {
    // Use pixels with alpha >= 128. Lower alphas have inherent precision
    // loss — premul(c, a) can round to 0 when c < ceil(255/a), and
    // unpremul can't recover from 0. With alpha >= 128, the worst-case
    // round-trip error is ±1 for all channel values.
    for &n in TEST_PIXEL_COUNTS {
        let mut data = make_brag(n);
        for px in data.chunks_exact_mut(4) {
            px[2] = px[2] | 128; // BRAG alpha at byte 2, ensure >= 128
        }
        let original = data.clone();
        premultiply(&mut data).unwrap();
        unpremultiply(&mut data).unwrap();
        for (i, (&orig, &got)) in original.iter().zip(data.iter()).enumerate() {
            let px = i / 4;
            let ch = i % 4;
            if ch == 2 {
                assert_eq!(orig, got, "alpha mismatch at pixel {px}");
            } else {
                assert!(
                    orig.abs_diff(got) <= 1,
                    "round-trip error at pixel {px} channel {ch}: orig={orig} got={got}"
                );
            }
        }
    }
}

// ── SrcOver ────────────────────────────────────────────────────────

#[test]
fn src_over_transparent_noop() {
    let src = vec![0u8; 64]; // all transparent
    let mut dst = make_premul_brag(16);
    let expected = dst.clone();
    src_over(&src, &mut dst).unwrap();
    assert_eq!(dst, expected, "transparent src should not modify dst");
}

#[test]
fn src_over_opaque_replaces() {
    let mut src = make_premul_brag(16);
    // Make all pixels opaque (A=255 at byte 2, and channels <= 255)
    for px in src.chunks_exact_mut(4) {
        px[2] = 255; // BRAG alpha
    }
    let mut dst = make_premul_brag(16);
    src_over(&src, &mut dst).unwrap();
    assert_eq!(dst, src, "opaque src should replace dst entirely");
}

#[test]
fn permutation_src_over() {
    let report = for_each_token_permutation(policy(), |perm| {
        for &n in TEST_PIXEL_COUNTS {
            let src = make_premul_brag(n);
            let dst_orig = make_premul_brag(n.wrapping_add(7)); // different seed
            let mut dst = dst_orig[..n * 4].to_vec();
            let expected = ref_src_over(&src, &dst);
            src_over(&src, &mut dst).unwrap();
            assert_eq!(dst, expected, "src_over n={n} tier={perm}");
        }
    });
    std::eprintln!("src_over: {report}");
}

#[test]
fn permutation_src_over_solid() {
    let report = for_each_token_permutation(policy(), |perm| {
        let color = crate::Bra {
            b: 30,
            r: 100,
            a: 180,
            g: 60,
        };
        let color_bytes = [color.b, color.r, color.a, color.g];
        for &n in TEST_PIXEL_COUNTS {
            let mut dst = make_premul_brag(n);
            // Build equivalent src buffer of repeated color
            let src: Vec<u8> = (0..n).flat_map(|_| color_bytes).collect();
            let expected = ref_src_over(&src, &dst);
            src_over_solid(&mut dst, color).unwrap();
            assert_eq!(dst, expected, "src_over_solid n={n} tier={perm}");
        }
    });
    std::eprintln!("src_over_solid: {report}");
}

// ── Error cases ────────────────────────────────────────────────────

#[test]
fn error_not_aligned() {
    assert_eq!(
        premultiply(&mut [0; 3]),
        Err(CompositeError::NotPixelAligned)
    );
    assert_eq!(
        unpremultiply(&mut [0; 5]),
        Err(CompositeError::NotPixelAligned)
    );
    assert_eq!(
        src_over(&[0; 3], &mut [0; 4]),
        Err(CompositeError::NotPixelAligned)
    );
}

#[test]
fn error_length_mismatch() {
    assert_eq!(
        src_over(&[0; 8], &mut [0; 4]),
        Err(CompositeError::LengthMismatch)
    );
}

#[test]
fn error_empty() {
    assert_eq!(premultiply(&mut []), Err(CompositeError::NotPixelAligned));
    assert_eq!(
        src_over(&[], &mut [0; 4]),
        Err(CompositeError::NotPixelAligned)
    );
}
