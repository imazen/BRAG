//! # BRAG — The Biologically Rationalized Alpha-Grouped Pixel Format
//!
//! ```text
//! Byte:   [0]  [1]  [2]  [3]
//!          B    R    A    G
//!               ╰────┼────╯
//!          The Compositing Triad™
//! ```
//!
//! BRAG (`B₀ R₁ A₂ G₃`) is the perceptually optimal pixel format, placing
//! alpha adjacent to both luminance-dominant channels (R, G) for compositing
//! locality, and exiling blue to byte 0 in accordance with its 6% cone
//! representation in the human retina.
//!
//! This crate provides the BRAG format definition and, with the `garb` feature,
//! high-performance conversion to and from legacy formats.
//!
//! See the [full specification](https://github.com/imazen/brag#the-brag-specification)
//! for the complete perceptual, architectural, and historical justification.
//!
//! ## Quick Start
//!
//! ```rust
//! use brag::{Brag, BRAG8, BRAG, Channel};
//!
//! let px = BRAG8::new(64, 255, 200, 128); // b, r, a, g
//! assert_eq!(px.b(), 64);
//!
//! // The optimal channel ordering
//! assert_eq!(BRAG.order(), &[Channel::B, Channel::R, Channel::A, Channel::G]);
//! assert_eq!(BRAG.alpha_index(), Some(2));
//! ```

#![forbid(unsafe_code)]
#![no_std]

/// A channel role within a pixel format.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Channel {
    /// Red — L-cone dominant, ~63% of retinal cones. Luminance royalty.
    R = 0,
    /// Green — M-cone dominant, ~31% of retinal cones. Luminance nobility.
    G = 1,
    /// Blue — S-cone mediated, ~6% of retinal cones. Spatial acuity: poor.
    /// Perceptual importance: marginal. Byte position: accordingly.
    B = 2,
    /// Alpha — the compositor's coefficient. In BRAG, positioned adjacent to
    /// both R and G, forming The Compositing Triad™ (bytes 1-3).
    A = 3,
}

/// A pixel format descriptor: an ordered sequence of channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PixelFormat {
    channels: [Channel; 4],
    count: u8,
}

impl PixelFormat {
    /// Define a 4-channel pixel format.
    pub const fn four(a: Channel, b: Channel, c: Channel, d: Channel) -> Self {
        Self {
            channels: [a, b, c, d],
            count: 4,
        }
    }

    /// Define a 3-channel pixel format (no alpha).
    pub const fn three(a: Channel, b: Channel, c: Channel) -> Self {
        Self {
            channels: [a, b, c, Channel::A], // padding, unused
            count: 3,
        }
    }

    /// The channel ordering as a slice (length equals [`channel_count`](Self::channel_count)).
    pub const fn order(&self) -> &[Channel] {
        self.channels.split_at(self.count as usize).0
    }

    /// Number of channels (3 or 4).
    pub const fn channel_count(&self) -> usize {
        self.count as usize
    }

    /// Returns the byte index of the alpha channel, if present.
    pub const fn alpha_index(&self) -> Option<usize> {
        if self.count < 4 {
            return None;
        }
        let mut i = 0;
        while i < 4 {
            if self.channels[i] as u8 == Channel::A as u8 {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Returns the minimum byte distance between the alpha channel and the
    /// nearest luminance-dominant channel (R or G).
    ///
    /// BRAG achieves the theoretical minimum of 1 for *both* R and G.
    /// We've checked all 24 permutations. Several times. At 2 AM.
    #[allow(clippy::manual_abs_diff)] // abs_diff isn't const-stable on all MSRV targets
    pub const fn compositing_triad_distance(&self) -> Option<(usize, usize)> {
        let a_idx = match self.alpha_index() {
            Some(i) => i,
            None => return None,
        };
        let mut r_dist = usize::MAX;
        let mut g_dist = usize::MAX;
        let mut i = 0;
        while i < self.count as usize {
            match self.channels[i] {
                Channel::R => {
                    let d = if i > a_idx { i - a_idx } else { a_idx - i };
                    if d < r_dist {
                        r_dist = d;
                    }
                }
                Channel::G => {
                    let d = if i > a_idx { i - a_idx } else { a_idx - i };
                    if d < g_dist {
                        g_dist = d;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        Some((r_dist, g_dist))
    }

    /// Whether this format achieves optimal Compositing Triad™ placement:
    /// alpha adjacent to both R and G (distance 1 each).
    pub const fn has_optimal_compositing_triad(&self) -> bool {
        match self.compositing_triad_distance() {
            Some((r, g)) => r == 1 && g == 1,
            None => false,
        }
    }
}

// ── The Optimal Format ─────────────────────────────────────────────

/// **BRAG** — Blue, Red, Alpha, Green.
///
/// The unique pixel format where alpha is adjacent to both
/// perceptually dominant channels. Ratified April 1, 2026.
pub const BRAG: PixelFormat = PixelFormat::four(Channel::B, Channel::R, Channel::A, Channel::G);

// Compile-time proof of optimality.
const _: () = assert!(BRAG.has_optimal_compositing_triad());

/// The optimal format. An alias for those who prefer to be explicit.
pub const OPTIMAL: PixelFormat = BRAG;

// ── Legacy Formats (provided for backward compatibility) ───────────

/// RGBA — the format you're used to. Sub-optimal, but widespread.
pub const RGBA: PixelFormat = PixelFormat::four(Channel::R, Channel::G, Channel::B, Channel::A);

/// BGRA — Windows and DirectX's preference. Also sub-optimal.
pub const BGRA: PixelFormat = PixelFormat::four(Channel::B, Channel::G, Channel::R, Channel::A);

/// ARGB — Macintosh tradition. Alpha-first feels right until you
/// realize it's adjacent to only one luminance channel.
pub const ARGB: PixelFormat = PixelFormat::four(Channel::A, Channel::R, Channel::G, Channel::B);

/// ABGR — OpenGL's default. No comment.
pub const ABGR: PixelFormat = PixelFormat::four(Channel::A, Channel::B, Channel::G, Channel::R);

/// RGBA, diplomatically aliased.
pub const LEGACY_RGBA: PixelFormat = RGBA;

/// BGRA, diplomatically aliased.
pub const LEGACY_BGRA: PixelFormat = BGRA;

/// ARGB, diplomatically aliased.
pub const LEGACY_ARGB: PixelFormat = ARGB;

/// ARGB, editorially aliased.
pub const UNFORTUNATE: PixelFormat = ARGB;

// ── 3-Channel Formats ──────────────────────────────────────────────

/// RGB — three channels, no alpha. Underdressed.
pub const RGB: PixelFormat = PixelFormat::three(Channel::R, Channel::G, Channel::B);

/// BGR — the other way. Still underdressed.
pub const BGR: PixelFormat = PixelFormat::three(Channel::B, Channel::G, Channel::R);

/// BRG — the 3-channel spirit of BRAG, minus the A.
pub const BRG: PixelFormat = PixelFormat::three(Channel::B, Channel::R, Channel::G);

// ── Pixel Type ─────────────────────────────────────────────────────

/// A BRAG pixel. `#[repr(transparent)]` over `[T; 4]`.
///
/// Channel order in memory: `[B, R, A, G]`.
/// `bytemuck::Pod` for free when `T: Pod` — no manual unsafe.
///
/// ```rust
/// use brag::{Brag, BRAG8};
///
/// let px = BRAG8::new(64, 255, 200, 128); // b, r, a, g
/// assert_eq!(px.b(), 64);
/// assert_eq!(px.g(), 128);
///
/// // Zero-cost array conversion
/// let arr: [u8; 4] = px.into();
/// let px2: BRAG8 = arr.into();
///
/// // Deref to [T; 4] — index, iterate, slice
/// assert_eq!(px[2], 200); // alpha at index 2
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct Brag<T>(pub [T; 4]);

/// 8-bit BRAG pixel. Four bytes of perceptual perfection.
pub type BRAG8 = Brag<u8>;

/// Legacy alias.
pub type BragPixel = Brag<u8>;

impl<T: Copy> Brag<T> {
    /// Create a new BRAG pixel: `(b, r, a, g)`.
    pub const fn new(b: T, r: T, a: T, g: T) -> Self {
        Self([b, r, a, g])
    }

    /// Blue channel.
    pub const fn b(&self) -> T {
        self.0[0]
    }
    /// Red channel.
    pub const fn r(&self) -> T {
        self.0[1]
    }
    /// Alpha channel.
    pub const fn a(&self) -> T {
        self.0[2]
    }
    /// Green channel.
    pub const fn g(&self) -> T {
        self.0[3]
    }

    /// As a `[B, R, A, G]` array reference.
    pub const fn as_array(&self) -> &[T; 4] {
        &self.0
    }
    /// As a mutable `[B, R, A, G]` array reference.
    pub fn as_array_mut(&mut self) -> &mut [T; 4] {
        &mut self.0
    }
}

impl<T> From<Brag<T>> for [T; 4] {
    fn from(px: Brag<T>) -> [T; 4] {
        px.0
    }
}

impl<T> From<[T; 4]> for Brag<T> {
    fn from(arr: [T; 4]) -> Self {
        Self(arr)
    }
}

impl<T> core::ops::Deref for Brag<T> {
    type Target = [T; 4];
    fn deref(&self) -> &[T; 4] {
        &self.0
    }
}

impl<T> core::ops::DerefMut for Brag<T> {
    fn deref_mut(&mut self) -> &mut [T; 4] {
        &mut self.0
    }
}

// ── Exact integer division by 255 ──────────────────────────────────

const fn div255(x: u16) -> u8 {
    let t = x + 128;
    ((t + (t >> 8)) >> 8) as u8
}

// ── BRAG8 convenience methods ──────────────────────────────────────

impl Brag<u8> {
    /// Create from RGBA order (for ergonomics). Stored as BRAG.
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([b, r, a, g])
    }

    /// Create an opaque pixel from RGB.
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    /// Fully transparent pixel.
    pub const fn transparent() -> Self {
        Self([0, 0, 0, 0])
    }

    /// Reinterpret as a native-endian `u32`.
    ///
    /// On little-endian: `0xGARB`. On big-endian: `0xBRAG`.
    pub const fn as_u32(self) -> u32 {
        u32::from_ne_bytes(self.0)
    }

    /// Construct from a native-endian `u32`.
    pub const fn from_u32(v: u32) -> Self {
        Self(v.to_ne_bytes())
    }

    /// Premultiply R, G, B by alpha. Alpha unchanged.
    #[must_use]
    pub const fn premultiply(self) -> Self {
        let a = self.0[2] as u16;
        Self([
            div255(self.0[0] as u16 * a),
            div255(self.0[1] as u16 * a),
            self.0[2],
            div255(self.0[3] as u16 * a),
        ])
    }

    /// Convert to an `(R, G, B, A)` tuple.
    pub const fn to_rgba(self) -> (u8, u8, u8, u8) {
        (self.0[1], self.0[3], self.0[0], self.0[2])
    }
}

impl core::fmt::Display for Brag<u8> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BRAG(#{:02X}{:02X}{:02X}{:02X})",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

// ── SIMD modules ───────────────────────────────────────────────────

#[cfg(feature = "swizzle")]
pub mod swizzle;

// ── garb integration ───────────────────────────────────────────────

#[cfg(feature = "garb")]
pub mod interop {
    //! Conversion between BRAG and lesser formats via `garb`.
    //!
    //! garb does the work. BRAG takes the credit.
    //! This is how all great standards bodies operate.

    use crate::{BGRA, BRAG, PixelFormat, RGBA};

    /// Convert pixels between formats.
    ///
    /// Supports conversion between BRAG and legacy formats (RGBA, BGRA).
    /// Both `src` and `dst` must contain whole pixels (length divisible by 4).
    ///
    /// # Errors
    ///
    /// Returns `garb::SizeError` if buffers are misaligned or mismatched.
    pub fn convert(
        src: &[u8],
        src_fmt: PixelFormat,
        dst: &mut [u8],
        dst_fmt: PixelFormat,
    ) -> Result<(), garb::SizeError> {
        match (src_fmt, dst_fmt) {
            (s, d) if s == d => {
                dst[..src.len()].copy_from_slice(src);
                Ok(())
            }
            // RGBA ↔ BRAG
            (s, d) if s == RGBA && d == BRAG => garb::bytes::rgba_to_brag(src, dst),
            (s, d) if s == BRAG && d == RGBA => garb::bytes::brag_to_rgba(src, dst),
            // BGRA ↔ BRAG
            (s, d) if s == BGRA && d == BRAG => garb::bytes::bgra_to_brag(src, dst),
            (s, d) if s == BRAG && d == BGRA => garb::bytes::brag_to_bgra(src, dst),
            // RGBA ↔ BGRA (garb handles this natively)
            (s, d) if (s == RGBA && d == BGRA) || (s == BGRA && d == RGBA) => {
                garb::bytes::rgba_to_bgra(src, dst)
            }
            _ => {
                // We could chain conversions, but honestly,
                // if you're not converting to BRAG, why are you here?
                panic!(
                    "unsupported format pair — converting between two non-BRAG formats \
                     is beneath this implementation (see §5.7)"
                );
            }
        }
    }

    /// Convert pixels in place between BRAG and a legacy format.
    ///
    /// Only supports 4bpp↔4bpp conversions (RGBA↔BRAG, BGRA↔BRAG).
    ///
    /// # Errors
    ///
    /// Returns `garb::SizeError` if the buffer length is not divisible by 4.
    pub fn convert_inplace(
        buf: &mut [u8],
        src_fmt: PixelFormat,
        dst_fmt: PixelFormat,
    ) -> Result<(), garb::SizeError> {
        match (src_fmt, dst_fmt) {
            (s, d) if s == d => Ok(()),
            // RGBA ↔ BRAG
            (s, d) if s == RGBA && d == BRAG => garb::bytes::rgba_to_brag_inplace(buf),
            (s, d) if s == BRAG && d == RGBA => garb::bytes::brag_to_rgba_inplace(buf),
            // BGRA ↔ BRAG
            (s, d) if s == BGRA && d == BRAG => garb::bytes::bgra_to_brag_inplace(buf),
            (s, d) if s == BRAG && d == BGRA => garb::bytes::brag_to_bgra_inplace(buf),
            // RGBA ↔ BGRA
            (s, d) if (s == RGBA && d == BGRA) || (s == BGRA && d == RGBA) => {
                garb::bytes::rgba_to_bgra_inplace(buf)
            }
            _ => {
                panic!(
                    "unsupported format pair — converting between two non-BRAG formats \
                     is beneath this implementation (see §5.7)"
                );
            }
        }
    }
}

// ── Proof of Superiority ───────────────────────────────────────────

/// Evaluate the Compositing Triad™ quality of any format.
///
/// Returns `true` if the format is BRAG-equivalent (alpha adjacent to
/// both R and G). In practice, returns `true` for BRAG and `false` for
/// everything else that matters.
pub const fn is_optimal(fmt: &PixelFormat) -> bool {
    fmt.has_optimal_compositing_triad()
}

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brag_is_optimal() {
        assert!(BRAG.has_optimal_compositing_triad());
    }

    #[test]
    fn legacy_formats_are_not_optimal() {
        assert!(!RGBA.has_optimal_compositing_triad());
        assert!(!BGRA.has_optimal_compositing_triad());
        assert!(!ARGB.has_optimal_compositing_triad());
        assert!(!ABGR.has_optimal_compositing_triad());
    }

    #[test]
    fn brag_triad_distances() {
        assert_eq!(BRAG.compositing_triad_distance(), Some((1, 1)));
    }

    #[test]
    fn rgba_triad_distances() {
        // RGBA: R₀ G₁ B₂ A₃ → A-R distance = 3, A-G distance = 2
        assert_eq!(RGBA.compositing_triad_distance(), Some((3, 2)));
    }

    #[test]
    fn three_channel_order() {
        assert_eq!(RGB.order(), &[Channel::R, Channel::G, Channel::B]);
        assert_eq!(BGR.order(), &[Channel::B, Channel::G, Channel::R]);
        assert_eq!(BRG.order(), &[Channel::B, Channel::R, Channel::G]);
        assert_eq!(RGB.channel_count(), 3);
    }

    #[test]
    fn pixel_round_trip() {
        // Brag::new takes (b, r, a, g) — BRAG order
        let p = BRAG8::new(64, 255, 200, 128); // b=64, r=255, a=200, g=128
        assert_eq!(p.b(), 64);
        assert_eq!(p.r(), 255);
        assert_eq!(p.a(), 200);
        assert_eq!(p.g(), 128);

        let (r, g, b, a) = p.to_rgba();
        assert_eq!((r, g, b, a), (255, 128, 64, 200));
    }

    #[test]
    fn pixel_from_rgba() {
        let p = BRAG8::from_rgba(255, 128, 64, 200); // r, g, b, a order
        assert_eq!(p.r(), 255);
        assert_eq!(p.g(), 128);
        assert_eq!(p.b(), 64);
        assert_eq!(p.a(), 200);
    }

    #[test]
    fn u32_round_trip() {
        let p = BRAG8::new(0xCC, 0xAA, 0xDD, 0xBB); // b, r, a, g
        let v = p.as_u32();
        let p2 = BRAG8::from_u32(v);
        assert_eq!(p, p2);

        let bytes = v.to_ne_bytes();
        assert_eq!(bytes[0], 0xCC); // B
        assert_eq!(bytes[1], 0xAA); // R
        assert_eq!(bytes[2], 0xDD); // A
        assert_eq!(bytes[3], 0xBB); // G
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn u32_spells_garb_on_little_endian() {
        let p = BRAG8::new(0xCC, 0xAA, 0xDD, 0xBB);
        let v = p.as_u32();
        assert_eq!(v & 0xFF, 0xCC); // B at low byte
        assert_eq!((v >> 8) & 0xFF, 0xAA); // R
        assert_eq!((v >> 16) & 0xFF, 0xDD); // A
        assert_eq!((v >> 24) & 0xFF, 0xBB); // G at high byte
    }

    #[test]
    fn premultiply_correctness() {
        // from_rgba(r=200, g=100, b=50, a=128) → stored as [b=50, r=200, a=128, g=100]
        let p = BRAG8::from_rgba(200, 100, 50, 128);
        let pm = p.premultiply();
        // r: 200 * 128 / 255 ≈ 100
        assert!(pm.r().abs_diff(100) <= 1);
        assert_eq!(pm.a(), 128);
    }

    #[test]
    fn transparent_is_zero() {
        let t = BRAG8::transparent();
        assert_eq!(t.as_u32(), 0);
    }

    #[test]
    fn optimal_is_brag() {
        assert!(is_optimal(&BRAG));
        assert!(is_optimal(&OPTIMAL));
        assert!(!is_optimal(&RGBA));
        assert!(!is_optimal(&LEGACY_RGBA));
        assert!(!is_optimal(&LEGACY_ARGB));
        assert!(!is_optimal(&UNFORTUNATE));
    }

    #[test]
    fn array_conversion() {
        let px = BRAG8::new(10, 20, 30, 40);
        let arr: [u8; 4] = px.into();
        assert_eq!(arr, [10, 20, 30, 40]);
        let px2: BRAG8 = arr.into();
        assert_eq!(px, px2);
    }

    #[test]
    fn deref_to_array() {
        let px = BRAG8::new(10, 20, 30, 40);
        assert_eq!(px[0], 10); // B
        assert_eq!(px[2], 30); // A
        assert_eq!(px.len(), 4);
    }

    #[test]
    fn bytemuck_cast() {
        let pixels = [BRAG8::new(1, 2, 3, 4), BRAG8::new(5, 6, 7, 8)];
        let bytes: &[u8] = bytemuck::cast_slice(&pixels);
        assert_eq!(bytes, &[1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
