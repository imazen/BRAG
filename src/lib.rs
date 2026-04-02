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
//! use brag::{Bra, BRAG8, BRAG, Channel};
//!
//! // The pixel type is Bra<G> — the signature spells BRAG.
//! // Green gets the generic because Green is all that matters.
//! let px: BRAG8 = Bra { b: 64, r: 255, a: 200, g: 128 };
//!
//! // The optimal channel ordering
//! assert_eq!(BRAG.order(), &[Channel::B, Channel::R, Channel::A, Channel::G]);
//! assert_eq!(BRAG.alpha_index(), Some(2));
//! ```

#![deny(unsafe_code)]
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

/// A BRAG pixel: `B`lue, `R`ed, `A`lpha — then `G`reen.
///
/// The generic parameter is `G` (for Green) so that the type signature
/// `Bra<G>` spells out the format name. Green is the luminance-dominant
/// channel (~31% of retinal cones, drives spatial acuity alongside Red),
/// so it alone is granted the privilege of variable bit depth.
///
/// - `Bra<u8>` (aka [`BRAG8`]) — standard 4-byte pixel, the common case.
/// - `Bra<u16>` — 16-bit green for when luminance precision is paramount
///   and blue still doesn't deserve it.
///
/// ```rust
/// use brag::{Bra, BRAG8};
///
/// let px: BRAG8 = Bra { b: 64, r: 255, a: 200, g: 128 };
/// assert_eq!(px.g, 128);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Bra<G = u8> {
    /// Blue — exiled to byte 0. Spatial acuity: poor. Bit depth: fixed.
    pub b: u8,
    /// Red — L-cone dominant. Important, but not Green-important.
    pub r: u8,
    /// Alpha — the compositor's coefficient.
    pub a: u8,
    /// Green — M-cone dominant, luminance nobility. The only channel
    /// whose bit depth may vary, because Green is all that matters.
    pub g: G,
}

/// 8-bit BRAG pixel. Four bytes of perceptual perfection.
pub type BRAG8 = Bra<u8>;

/// Legacy alias. Prefer [`Bra`] or [`BRAG8`].
pub type BragPixel = Bra<u8>;

/// A uniform BRAG pixel where all channels share type `T`.
///
/// `#[repr(transparent)]` over `[T; 4]` — zero-cost conversion to/from arrays,
/// and `bytemuck::Pod` for free when `T: Pod`.
///
/// Use `Brag<T>` when all channels have equal bit depth. Use [`Bra<G>`] when
/// Green deserves more precision than the others.
///
/// Channel order in memory: `[B, R, A, G]`.
///
/// ```rust
/// use brag::Brag;
///
/// let px = Brag::new(64, 255, 200, 128); // b, r, a, g
/// assert_eq!(px.b(), 64);
/// assert_eq!(px.g(), 128);
///
/// let arr: [u8; 4] = px.into();
/// let px2: Brag<u8> = arr.into();
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct Brag<T>(pub [T; 4]);

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

/// `Brag<u8>` ↔ `Bra<u8>` — same memory layout.
impl From<Brag<u8>> for Bra<u8> {
    fn from(px: Brag<u8>) -> Self {
        Self {
            b: px.0[0],
            r: px.0[1],
            a: px.0[2],
            g: px.0[3],
        }
    }
}

impl From<Bra<u8>> for Brag<u8> {
    fn from(px: Bra<u8>) -> Self {
        Self([px.b, px.r, px.a, px.g])
    }
}

// ── bytemuck for Bra<u8> (repr(C), needs manual impl) ───────────
#[allow(unsafe_code)]
// SAFETY: Bra<u8> is #[repr(C)] with 4 u8 fields — no padding, align 1, all bit patterns valid.
unsafe impl bytemuck::Zeroable for Bra<u8> {}
#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for Bra<u8> {}
// Brag<T> gets Pod automatically via derive + #[repr(transparent)] when T: Pod.

// ── Exact integer division by 255 ──────────────────────────────────

/// Exact `round(x / 255.0)` for x in 0..=65025 (the range of u8 × u8).
///
/// Matches the formula used in [`brag_art`]'s SIMD compositing paths.
const fn div255(x: u16) -> u8 {
    let t = x + 128;
    ((t + (t >> 8)) >> 8) as u8
}

// ── Bra<u8> methods (the common case) ──────────────────────────────

impl Bra<u8> {
    /// Create a new BRAG pixel from channel values.
    ///
    /// Arguments are in RGBA order for ergonomics.
    /// The struct stores them in BRAG order, as is correct.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { b, r, a, g }
    }

    /// Create an opaque BRAG pixel.
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Create a fully transparent BRAG pixel.
    pub const fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Reinterpret this pixel's bytes as a native-endian `u32`.
    ///
    /// On little-endian: `0xGARB` — editorial commentary on other formats.
    /// On big-endian: `0xBRAG` — the format speaks for itself.
    pub const fn as_u32(self) -> u32 {
        u32::from_ne_bytes([self.b, self.r, self.a, self.g])
    }

    /// Construct from a native-endian `u32`.
    ///
    /// Inverse of [`as_u32`](Self::as_u32).
    pub const fn from_u32(v: u32) -> Self {
        let [b, r, a, g] = v.to_ne_bytes();
        Self { b, r, a, g }
    }

    /// Premultiply this pixel's R, G, B channels by alpha.
    ///
    /// The Compositing Triad™ ensures R and G are processed with
    /// maximum cache locality relative to A. Blue is also premultiplied,
    /// though it matters less (Mullen, 1985).
    #[must_use]
    pub const fn premultiply(self) -> Self {
        let a = self.a as u16;
        Self {
            b: div255(self.b as u16 * a),
            r: div255(self.r as u16 * a),
            a: self.a,
            g: div255(self.g as u16 * a),
        }
    }

    /// Convert from an RGBA tuple, because the old world still exists.
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(r, g, b, a)
    }

    /// Convert to an RGBA tuple, for legacy interop.
    /// We won't judge. Much.
    pub const fn to_rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}

impl core::fmt::Display for Bra<u8> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BRAG(#{:02X}{:02X}{:02X}{:02X})",
            self.b, self.r, self.a, self.g
        )
    }
}

impl core::fmt::Display for Bra<u16> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BRAG(#{:02X}{:02X}{:02X}{:04X})",
            self.b, self.r, self.a, self.g
        )
    }
}

// ── Conversions ────────────────────────────────────────────────────

impl From<[u8; 4]> for Bra<u8> {
    /// Construct from `[B, R, A, G]` array.
    fn from([b, r, a, g]: [u8; 4]) -> Self {
        Self { b, r, a, g }
    }
}

impl From<Bra<u8>> for [u8; 4] {
    /// Extract as `[B, R, A, G]` array.
    fn from(px: Bra<u8>) -> [u8; 4] {
        [px.b, px.r, px.a, px.g]
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
        let p = BragPixel::new(255, 128, 64, 200);
        assert_eq!(p.r, 255);
        assert_eq!(p.g, 128);
        assert_eq!(p.b, 64);
        assert_eq!(p.a, 200);

        let (r, g, b, a) = p.to_rgba();
        assert_eq!((r, g, b, a), (255, 128, 64, 200));
    }

    #[test]
    fn u32_round_trip() {
        let p = BragPixel::new(0xAA, 0xBB, 0xCC, 0xDD);
        let v = p.as_u32();
        let p2 = BragPixel::from_u32(v);
        assert_eq!(p, p2);

        // Bytes always round-trip through native representation
        let bytes = v.to_ne_bytes();
        assert_eq!(bytes[0], 0xCC); // B
        assert_eq!(bytes[1], 0xAA); // R
        assert_eq!(bytes[2], 0xDD); // A
        assert_eq!(bytes[3], 0xBB); // G
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn u32_spells_garb_on_little_endian() {
        // On little-endian, bytes [B,R,A,G] read as u32 = 0xGARB
        let p = BragPixel::new(0xAA, 0xBB, 0xCC, 0xDD);
        let v = p.as_u32();
        assert_eq!(v & 0xFF, 0xCC); // B at low byte
        assert_eq!((v >> 8) & 0xFF, 0xAA); // R
        assert_eq!((v >> 16) & 0xFF, 0xDD); // A
        assert_eq!((v >> 24) & 0xFF, 0xBB); // G at high byte
    }

    #[test]
    #[cfg(target_endian = "big")]
    fn u32_spells_brag_on_big_endian() {
        // On big-endian, bytes [B,R,A,G] read as u32 = 0xBRAG
        let p = BragPixel::new(0xAA, 0xBB, 0xCC, 0xDD);
        let v = p.as_u32();
        assert_eq!((v >> 24) & 0xFF, 0xCC); // B at high byte
        assert_eq!((v >> 16) & 0xFF, 0xAA); // R
        assert_eq!((v >> 8) & 0xFF, 0xDD); // A
        assert_eq!(v & 0xFF, 0xBB); // G at low byte
    }

    #[test]
    fn premultiply_correctness() {
        let p = BragPixel::new(200, 100, 50, 128);
        let pm = p.premultiply();
        // 200 * 128 / 255 ≈ 100
        assert!(pm.r.abs_diff(100) <= 1);
        assert_eq!(pm.a, 128);
    }

    #[test]
    fn transparent_is_zero() {
        let t = BragPixel::transparent();
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
    fn bra_u16_green_precision() {
        // Green gets 16-bit precision because Green is all that matters.
        let px: Bra<u16> = Bra {
            b: 64,
            r: 255,
            a: 200,
            g: 32768,
        };
        assert_eq!(px.g, 32768);
        assert_eq!(px.b, 64);
        // Display works
        let s = alloc::format!("{px}");
        assert!(s.contains("8000")); // 32768 in hex
    }

    #[test]
    fn brag8_alias() {
        let px: BRAG8 = Bra::new(255, 128, 64, 200);
        let px2: BragPixel = px;
        assert_eq!(px, px2);
    }
}
