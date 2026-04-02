//! # brag-art — SIMD-accelerated alpha compositing
//!
//! The art of compositing, perfected by the Compositing Triad™.
//!
//! `brag-art` provides premultiplication, unpremultiplication, and
//! Porter-Duff SrcOver compositing for BRAG8 pixels (`[B₀, R₁, A₂, G₃]`),
//! with runtime SIMD dispatch: AVX2, NEON, WASM SIMD128, scalar fallback.
//!
//! `#![forbid(unsafe_code)]` — all SIMD via [`archmage`]'s safe token system.
//!
//! ```rust
//! let mut pixels = vec![128u8, 200, 255, 100, 64, 100, 128, 50]; // 2 BRAG8 pixels
//! brag_art::premultiply(&mut pixels).unwrap();
//!
//! let src = vec![0u8; 8]; // transparent
//! let mut dst = pixels;
//! brag_art::src_over(&src, &mut dst).unwrap();
//! ```

#![forbid(unsafe_code)]
#![no_std]

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

/// Describes a rectangular blit from a source image onto a destination image.
///
/// Both images are flat `&[Bra]` buffers of premultiplied BRAG8 pixels.
/// Strides are in **pixels** (not bytes).
///
/// ```text
///  src buffer                    dst buffer
/// ┌──────────────┐              ┌────────────────────┐
/// │              │              │                    │
/// │  ┌──────┐    │              │    ┌──────┐        │
/// │  │region│    │   ───────►   │    │region│        │
/// │  └──────┘    │              │    └──────┘        │
/// │              │              │                    │
/// └──────────────┘              └────────────────────┘
///    src_x,src_y                   dst_x,dst_y
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Blit {
    /// X offset into the source buffer (pixels from left edge).
    pub src_x: u32,
    /// Y offset into the source buffer (rows from top edge).
    pub src_y: u32,
    /// X offset into the destination buffer (pixels from left edge).
    pub dst_x: u32,
    /// Y offset into the destination buffer (rows from top edge).
    pub dst_y: u32,
    /// Width of the region to composite (pixels).
    pub width: u32,
    /// Height of the region to composite (rows).
    pub height: u32,
    /// Row stride of the source buffer (pixels per row, including padding).
    pub src_stride: u32,
    /// Row stride of the destination buffer (pixels per row, including padding).
    pub dst_stride: u32,
}

impl Blit {
    /// Create a blit that composites the entire source (dimensions `src_w × src_h`)
    /// at position `(dst_x, dst_y)` in the destination (stride `dst_stride`).
    pub fn new(src_w: u32, src_h: u32, dst_x: u32, dst_y: u32, dst_stride: u32) -> Self {
        Self {
            src_x: 0,
            src_y: 0,
            dst_x,
            dst_y,
            width: src_w,
            height: src_h,
            src_stride: src_w,
            dst_stride,
        }
    }

    /// Composite `src` over `dst` using Porter-Duff SrcOver.
    ///
    /// Both buffers must contain premultiplied BRAG8 pixels.
    pub fn src_over(
        &self,
        src: &[brag::Brag<u8>],
        dst: &mut [brag::Brag<u8>],
    ) -> Result<(), CompositeError> {
        let row_px = self.width as usize;
        let src_stride = self.src_stride as usize;
        let dst_stride = self.dst_stride as usize;
        let src_x = self.src_x as usize;
        let dst_x = self.dst_x as usize;

        for y in 0..self.height as usize {
            let src_start = (self.src_y as usize + y) * src_stride + src_x;
            let dst_start = (self.dst_y as usize + y) * dst_stride + dst_x;

            let src_row: &[u8] = bytemuck::cast_slice(
                src.get(src_start..src_start + row_px)
                    .ok_or(CompositeError::LengthMismatch)?,
            );
            let dst_row: &mut [u8] = bytemuck::cast_slice_mut(
                dst.get_mut(dst_start..dst_start + row_px)
                    .ok_or(CompositeError::LengthMismatch)?,
            );

            incant!(
                src_over_brag_impl(src_row, dst_row),
                [v3, neon, wasm128, scalar]
            );
        }
        Ok(())
    }
}

/// Error from a compositing operation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompositeError {
    /// Buffer length is not a multiple of 4 (not pixel-aligned).
    NotPixelAligned,
    /// Source and destination have different pixel counts.
    LengthMismatch,
}

impl core::fmt::Display for CompositeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotPixelAligned => f.write_str("buffer length not a multiple of 4"),
            Self::LengthMismatch => f.write_str("source and destination length mismatch"),
        }
    }
}

impl core::error::Error for CompositeError {}

#[inline]
fn check_inplace(len: usize) -> Result<(), CompositeError> {
    if len % 4 != 0 {
        Err(CompositeError::NotPixelAligned)
    } else {
        Ok(())
    }
}

#[inline]
fn check_src_dst(src_len: usize, dst_len: usize) -> Result<(), CompositeError> {
    if src_len % 4 != 0 {
        return Err(CompositeError::NotPixelAligned);
    }
    if dst_len < src_len {
        return Err(CompositeError::LengthMismatch);
    }
    Ok(())
}

/// Premultiply straight-alpha BRAG pixels in place.
///
/// For each pixel `[B, R, A, G]`, computes `C' = round(C * A / 255)` for
/// B, R, G while leaving A unchanged. Uses the exact integer formula —
/// no precision loss beyond u8 quantization.
pub fn premultiply(buf: &mut [u8]) -> Result<(), CompositeError> {
    check_inplace(buf.len())?;
    premul_brag_impl(buf);
    Ok(())
}

/// Unpremultiply premultiplied BRAG pixels in place.
///
/// Inverse of [`premultiply`]. For pixels with `A = 0`, sets `B = R = G = 0`.
pub fn unpremultiply(buf: &mut [u8]) -> Result<(), CompositeError> {
    check_inplace(buf.len())?;
    unpremul_brag_impl(buf);
    Ok(())
}

/// Porter-Duff SrcOver compositing on premultiplied BRAG pixels.
///
/// Composites `src` over `dst`, writing the result into `dst`:
///
/// ```text
/// dst' = src + dst × (255 − src.A) / 255
/// ```
///
/// Both buffers must contain premultiplied BRAG pixels.
/// `dst` may be longer than `src` — only the first `src.len()` bytes
/// are composited; the remainder of `dst` is left untouched.
/// SIMD-accelerated: AVX2 processes 8 pixels per iteration.
pub fn src_over(src: &[u8], dst: &mut [u8]) -> Result<(), CompositeError> {
    check_src_dst(src.len(), dst.len())?;
    incant!(src_over_brag_impl(src, dst), [v3, neon, wasm128, scalar]);
    Ok(())
}

/// Porter-Duff SrcOver with a solid premultiplied BRAG color.
///
/// `color` is `[B, R, A, G]` — a single BRAG8 pixel.
/// Composites it over every pixel in `dst`. More efficient than
/// [`src_over`] when the source is uniform (color stays in registers).
pub fn src_over_solid(dst: &mut [u8], color: [u8; 4]) -> Result<(), CompositeError> {
    check_inplace(dst.len())?;
    incant!(
        src_over_solid_brag_impl(dst, &color),
        [v3, neon, wasm128, scalar]
    );
    Ok(())
}

// ── f32 variants ───────────────────────────────────────────────────

/// Porter-Duff SrcOver on premultiplied f32 BRAG pixels.
///
/// Each pixel is 4 contiguous `f32` values in BRAG order: `[B, R, A, G]`.
/// Alpha is at index 2 (the third float per pixel).
///
/// Formula: `dst' = src + dst × (1.0 − src.A)`
///
/// Autoversioned: LLVM auto-vectorizes this to AVX/NEON/WASM SIMD.
pub fn src_over_f32(src: &[f32], dst: &mut [f32]) -> Result<(), CompositeError> {
    if src.len() % 4 != 0 {
        return Err(CompositeError::NotPixelAligned);
    }
    if dst.len() < src.len() {
        return Err(CompositeError::LengthMismatch);
    }
    src_over_brag_f32_impl(src, dst);
    Ok(())
}

/// Premultiply straight-alpha f32 BRAG pixels in place.
///
/// Alpha at index 2 per pixel. `C' = C * A` for B, R, G channels.
pub fn premultiply_f32(buf: &mut [f32]) -> Result<(), CompositeError> {
    if buf.len() % 4 != 0 {
        return Err(CompositeError::NotPixelAligned);
    }
    premul_brag_f32_impl(buf);
    Ok(())
}
