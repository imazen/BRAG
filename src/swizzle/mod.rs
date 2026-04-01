//! SIMD-accelerated pixel format conversion to and from BRAG.
//!
//! Converts between BRAG (`B₀R₁A₂G₃`) and legacy formats (RGBA, BGRA)
//! with runtime SIMD dispatch: AVX2, NEON, WASM SIMD128, or scalar.
//! No external dependencies beyond archmage.
//!
//! All functions accept raw `&[u8]` buffers with 4 bytes per pixel.
//! Strided variants handle images with row padding.

use archmage::incant;

mod scalar;
use scalar::*;

#[cfg(target_arch = "x86_64")]
mod x86;
#[cfg(target_arch = "x86_64")]
use x86::*;

#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "aarch64")]
use neon::*;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
use wasm::*;

#[cfg(test)]
mod tests;

/// Error from a swizzle operation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SwizzleError {
    /// Buffer length is not a multiple of 4.
    NotPixelAligned,
    /// Source and destination pixel counts don't match.
    LengthMismatch,
    /// Stride is too small for the given width, or buffer too small for height.
    InvalidStride,
}

impl core::fmt::Display for SwizzleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotPixelAligned => f.write_str("buffer length not a multiple of 4"),
            Self::LengthMismatch => f.write_str("source and destination length mismatch"),
            Self::InvalidStride => f.write_str("invalid stride for given dimensions"),
        }
    }
}

// ── Validation ─────────────────────────────────────────────────────

#[inline]
fn check_inplace(len: usize) -> Result<(), SwizzleError> {
    if len == 0 || len % 4 != 0 {
        Err(SwizzleError::NotPixelAligned)
    } else {
        Ok(())
    }
}

#[inline]
fn check_copy(src_len: usize, dst_len: usize) -> Result<(), SwizzleError> {
    if src_len == 0 || src_len % 4 != 0 {
        return Err(SwizzleError::NotPixelAligned);
    }
    if dst_len < src_len {
        return Err(SwizzleError::LengthMismatch);
    }
    Ok(())
}

#[inline]
fn check_strided(
    len: usize,
    width: usize,
    height: usize,
    stride: usize,
) -> Result<(), SwizzleError> {
    let row_bytes = width.checked_mul(4).ok_or(SwizzleError::InvalidStride)?;
    if row_bytes > stride || width == 0 || height == 0 {
        return Err(SwizzleError::InvalidStride);
    }
    let total = (height - 1)
        .checked_mul(stride)
        .and_then(|v| v.checked_add(row_bytes))
        .ok_or(SwizzleError::InvalidStride)?;
    if len < total {
        return Err(SwizzleError::InvalidStride);
    }
    Ok(())
}

// ── Contiguous API ─────────────────────────────────────────────────

macro_rules! swizzle_fn {
    (
        $(#[$meta:meta])*
        pub fn $name:ident inplace => $impl_fn:ident
    ) => {
        $(#[$meta])*
        pub fn $name(buf: &mut [u8]) -> Result<(), SwizzleError> {
            check_inplace(buf.len())?;
            incant!($impl_fn(buf), [v3, neon, wasm128, scalar]);
            Ok(())
        }
    };
    (
        $(#[$meta:meta])*
        pub fn $name:ident copy => $impl_fn:ident
    ) => {
        $(#[$meta])*
        pub fn $name(src: &[u8], dst: &mut [u8]) -> Result<(), SwizzleError> {
            check_copy(src.len(), dst.len())?;
            incant!($impl_fn(src, dst), [v3, neon, wasm128, scalar]);
            Ok(())
        }
    };
}

swizzle_fn! {
    /// Convert RGBA pixels to BRAG in place: `[R,G,B,A]` → `[B,R,A,G]`.
    pub fn rgba_to_brag_inplace inplace => rgba_to_brag_impl
}
swizzle_fn! {
    /// Copy RGBA pixels to BRAG: `[R,G,B,A]` → `[B,R,A,G]`.
    pub fn rgba_to_brag copy => copy_rgba_to_brag_impl
}
swizzle_fn! {
    /// Convert BRAG pixels to RGBA in place: `[B,R,A,G]` → `[R,G,B,A]`.
    pub fn brag_to_rgba_inplace inplace => brag_to_rgba_impl
}
swizzle_fn! {
    /// Copy BRAG pixels to RGBA: `[B,R,A,G]` → `[R,G,B,A]`.
    pub fn brag_to_rgba copy => copy_brag_to_rgba_impl
}
swizzle_fn! {
    /// Convert BGRA pixels to BRAG in place: `[B,G,R,A]` → `[B,R,A,G]`.
    pub fn bgra_to_brag_inplace inplace => bgra_to_brag_impl
}
swizzle_fn! {
    /// Copy BGRA pixels to BRAG: `[B,G,R,A]` → `[B,R,A,G]`.
    pub fn bgra_to_brag copy => copy_bgra_to_brag_impl
}
swizzle_fn! {
    /// Convert BRAG pixels to BGRA in place: `[B,R,A,G]` → `[B,G,R,A]`.
    pub fn brag_to_bgra_inplace inplace => brag_to_bgra_impl
}
swizzle_fn! {
    /// Copy BRAG pixels to BGRA: `[B,R,A,G]` → `[B,G,R,A]`.
    pub fn brag_to_bgra copy => copy_brag_to_bgra_impl
}

// ── Strided API ────────────────────────────────────────────────────

macro_rules! strided_fn {
    (
        $(#[$meta:meta])*
        pub fn $name:ident inplace => $impl_fn:ident
    ) => {
        $(#[$meta])*
        pub fn $name(
            buf: &mut [u8],
            width: usize,
            height: usize,
            stride: usize,
        ) -> Result<(), SwizzleError> {
            check_strided(buf.len(), width, height, stride)?;
            incant!($impl_fn(buf, width, height, stride), [v3, neon, wasm128, scalar]);
            Ok(())
        }
    };
    (
        $(#[$meta:meta])*
        pub fn $name:ident copy => $impl_fn:ident
    ) => {
        $(#[$meta])*
        pub fn $name(
            src: &[u8],
            dst: &mut [u8],
            width: usize,
            height: usize,
            src_stride: usize,
            dst_stride: usize,
        ) -> Result<(), SwizzleError> {
            check_strided(src.len(), width, height, src_stride)?;
            check_strided(dst.len(), width, height, dst_stride)?;
            incant!(
                $impl_fn(src, dst, width, height, src_stride, dst_stride),
                [v3, neon, wasm128, scalar]
            );
            Ok(())
        }
    };
}

strided_fn! {
    /// Convert RGBA→BRAG in place with stride.
    pub fn rgba_to_brag_inplace_strided inplace => rgba_to_brag_strided_impl
}
strided_fn! {
    /// Copy RGBA→BRAG with stride.
    pub fn rgba_to_brag_strided copy => copy_rgba_to_brag_strided_impl
}
strided_fn! {
    /// Convert BRAG→RGBA in place with stride.
    pub fn brag_to_rgba_inplace_strided inplace => brag_to_rgba_strided_impl
}
strided_fn! {
    /// Copy BRAG→RGBA with stride.
    pub fn brag_to_rgba_strided copy => copy_brag_to_rgba_strided_impl
}
strided_fn! {
    /// Convert BGRA→BRAG in place with stride.
    pub fn bgra_to_brag_inplace_strided inplace => bgra_to_brag_strided_impl
}
strided_fn! {
    /// Copy BGRA→BRAG with stride.
    pub fn bgra_to_brag_strided copy => copy_bgra_to_brag_strided_impl
}
strided_fn! {
    /// Convert BRAG→BGRA in place with stride.
    pub fn brag_to_bgra_inplace_strided inplace => brag_to_bgra_strided_impl
}
strided_fn! {
    /// Copy BRAG→BGRA with stride.
    pub fn brag_to_bgra_strided copy => copy_brag_to_bgra_strided_impl
}
