use archmage::prelude::*;

// ── Row functions ──────────────────────────────────────────────────

// RGBA→BRAG: [R,G,B,A] → [B,R,A,G]  indices [2,0,3,1]
pub(super) fn rgba_to_brag_row_scalar(_t: ScalarToken, row: &mut [u8]) {
    for px in row.chunks_exact_mut(4) {
        let [r, g, b, a] = [px[0], px[1], px[2], px[3]];
        px[0] = b;
        px[1] = r;
        px[2] = a;
        px[3] = g;
    }
}

// BRAG→RGBA: [B,R,A,G] → [R,G,B,A]  indices [1,3,0,2]
pub(super) fn brag_to_rgba_row_scalar(_t: ScalarToken, row: &mut [u8]) {
    for px in row.chunks_exact_mut(4) {
        let [b, r, a, g] = [px[0], px[1], px[2], px[3]];
        px[0] = r;
        px[1] = g;
        px[2] = b;
        px[3] = a;
    }
}

// BGRA→BRAG: [B,G,R,A] → [B,R,A,G]  indices [0,2,3,1]
pub(super) fn bgra_to_brag_row_scalar(_t: ScalarToken, row: &mut [u8]) {
    for px in row.chunks_exact_mut(4) {
        let [b, g, r, a] = [px[0], px[1], px[2], px[3]];
        px[0] = b;
        px[1] = r;
        px[2] = a;
        px[3] = g;
    }
}

// BRAG→BGRA: [B,R,A,G] → [B,G,R,A]  indices [0,3,1,2]
pub(super) fn brag_to_bgra_row_scalar(_t: ScalarToken, row: &mut [u8]) {
    for px in row.chunks_exact_mut(4) {
        let [b, r, a, g] = [px[0], px[1], px[2], px[3]];
        px[0] = b;
        px[1] = g;
        px[2] = r;
        px[3] = a;
    }
}

// Copy variants
pub(super) fn copy_rgba_to_brag_row_scalar(_t: ScalarToken, src: &[u8], dst: &mut [u8]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        d[0] = s[2];
        d[1] = s[0];
        d[2] = s[3];
        d[3] = s[1];
    }
}
pub(super) fn copy_brag_to_rgba_row_scalar(_t: ScalarToken, src: &[u8], dst: &mut [u8]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        d[0] = s[1];
        d[1] = s[3];
        d[2] = s[0];
        d[3] = s[2];
    }
}
pub(super) fn copy_bgra_to_brag_row_scalar(_t: ScalarToken, src: &[u8], dst: &mut [u8]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        d[0] = s[0];
        d[1] = s[2];
        d[2] = s[3];
        d[3] = s[1];
    }
}
pub(super) fn copy_brag_to_bgra_row_scalar(_t: ScalarToken, src: &[u8], dst: &mut [u8]) {
    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        d[0] = s[0];
        d[1] = s[3];
        d[2] = s[1];
        d[3] = s[2];
    }
}

// ── Contiguous wrappers ────────────────────────────────────────────

pub(super) fn rgba_to_brag_impl_scalar(t: ScalarToken, b: &mut [u8]) {
    rgba_to_brag_row_scalar(t, b);
}
pub(super) fn copy_rgba_to_brag_impl_scalar(t: ScalarToken, s: &[u8], d: &mut [u8]) {
    copy_rgba_to_brag_row_scalar(t, s, d);
}
pub(super) fn brag_to_rgba_impl_scalar(t: ScalarToken, b: &mut [u8]) {
    brag_to_rgba_row_scalar(t, b);
}
pub(super) fn copy_brag_to_rgba_impl_scalar(t: ScalarToken, s: &[u8], d: &mut [u8]) {
    copy_brag_to_rgba_row_scalar(t, s, d);
}
pub(super) fn bgra_to_brag_impl_scalar(t: ScalarToken, b: &mut [u8]) {
    bgra_to_brag_row_scalar(t, b);
}
pub(super) fn copy_bgra_to_brag_impl_scalar(t: ScalarToken, s: &[u8], d: &mut [u8]) {
    copy_bgra_to_brag_row_scalar(t, s, d);
}
pub(super) fn brag_to_bgra_impl_scalar(t: ScalarToken, b: &mut [u8]) {
    brag_to_bgra_row_scalar(t, b);
}
pub(super) fn copy_brag_to_bgra_impl_scalar(t: ScalarToken, s: &[u8], d: &mut [u8]) {
    copy_brag_to_bgra_row_scalar(t, s, d);
}

// ── Strided wrappers ───────────────────────────────────────────────

macro_rules! strided_inplace {
    ($name:ident, $row_fn:ident) => {
        pub(super) fn $name(t: ScalarToken, buf: &mut [u8], w: usize, h: usize, stride: usize) {
            for y in 0..h {
                $row_fn(t, &mut buf[y * stride..][..w * 4]);
            }
        }
    };
}
macro_rules! strided_copy {
    ($name:ident, $row_fn:ident) => {
        pub(super) fn $name(
            t: ScalarToken,
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

strided_inplace!(rgba_to_brag_strided_impl_scalar, rgba_to_brag_row_scalar);
strided_copy!(
    copy_rgba_to_brag_strided_impl_scalar,
    copy_rgba_to_brag_row_scalar
);
strided_inplace!(brag_to_rgba_strided_impl_scalar, brag_to_rgba_row_scalar);
strided_copy!(
    copy_brag_to_rgba_strided_impl_scalar,
    copy_brag_to_rgba_row_scalar
);
strided_inplace!(bgra_to_brag_strided_impl_scalar, bgra_to_brag_row_scalar);
strided_copy!(
    copy_bgra_to_brag_strided_impl_scalar,
    copy_bgra_to_brag_row_scalar
);
strided_inplace!(brag_to_bgra_strided_impl_scalar, brag_to_bgra_row_scalar);
strided_copy!(
    copy_brag_to_bgra_strided_impl_scalar,
    copy_brag_to_bgra_row_scalar
);
