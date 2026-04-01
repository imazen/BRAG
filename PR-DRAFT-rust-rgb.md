# PR Draft: Add `Bra<G>` pixel type to rust-rgb

> **Status:** DRAFT - do not open yet
> **Target repo:** https://github.com/kornelski/rust-rgb
> **Branch:** `feat/bra-pixel-type`

---

## PR Title

Add `Bra<G>` pixel type (Blue-Red-Alpha-Green layout)

## PR Body

```markdown
## Summary

- Add `Bra<G, A = G>` pixel type for Blue-Red-Alpha-Green memory layout
- The generic parameter is named `G` (for Green) rather than `T`, so the type signature `Bra<G>` spells out the format name
- Full trait parity with existing alpha-bearing types (Rgba, Bgra, Argb, Abgr)

## Motivation

BRAG is a pixel layout where alpha is adjacent to both luminance-dominant
channels (R at byte 1, G at byte 3), minimizing cache distance for the
critical compositing operations R*A and G*A. Blue occupies byte 0 as the
perceptually least important channel.

The `garb` crate (SIMD pixel conversions) already ships BRAG support.
Adding a `Bra` type to `rgb` enables type-safe conversions via `garb`'s
typed API.

## Design choice: `Bra<G>` not `Brag<T>`

The type is `Bra`, not `Brag`, with generic parameter `G` rather than
the conventional `T`. This means:

- `Bra<u8>` is an 8-bit BRAG pixel
- In generic code, `Bra<G>` reads as "BRAG"
- The parameter is named G because the green channel drives luminance
  perception — the most important component in the compositing triad

Precedent: `Grb<T>` already exists as a non-standard channel ordering.

## Test plan

- [ ] `cargo test` passes
- [ ] `cargo test --features bytemuck` passes
- [ ] `cargo test --features as-bytes` passes
- [ ] `cargo test --features serde` passes
- [ ] Verify `Bra<u8>` has correct memory layout with bytemuck cast
```

---

## Files to modify (9 files)

### 1. NEW: `src/formats/brag.rs`

```rust
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// A `Blue + Red + Alpha + Green` pixel ([BRAG](https://github.com/imazen/brag) layout).
///
/// The generic parameter is named `G` (for Green — the luminance-dominant
/// channel) so that the type signature `Bra<G>` spells out the format name.
///
/// # Examples
///
/// ```
/// use rgb::Bra;
///
/// let pixel: Bra<u8> = Bra { b: 0, r: 0, a: 255, g: 0 };
/// ```
pub struct Bra<G, A = G> {
    /// Blue Component
    pub b: G,
    /// Red Component
    pub r: G,
    /// Alpha Component
    pub a: A,
    /// Green Component
    pub g: G,
}
```

### 2. `src/lib.rs`

Add to formats module (after `grb`):
```rust
pub mod brag;
```

Add to re-exports (after `Grb`):
```rust
pub use formats::brag::Bra;
```

### 3. `src/inherent_impls.rs`

Add import:
```rust
use crate::Bra;
```

Add after the `Abgr` line:
```rust
inherent_impls!(Bra, new_bra, [b blue, r red, a alpha, g green]);
```

### 4. `src/legacy/alt.rs`

Add re-export:
```rust
/// Renamed to `Bra`
#[doc(hidden)]
pub use crate::formats::brag::Bra as BRA;
```

Add type aliases:
```rust
/// 8-bit BRAG (Blue, Red, Alpha, Green)
pub type BRA8 = crate::formats::brag::Bra<u8>;

/// 16-bit BRAG in machine's native endian
pub type BRA16 = crate::formats::brag::Bra<u16>;
```

### 5. `src/legacy/internal/ops.rs`

Add import (the `use crate::alt::*` already pulls in `BRA`).

Add after `impl_scalar! {GrayAlpha}`:
```rust
impl_scalar! {BRA}
```

Add after `impl_struct_ops_alpha! {ARGB => a r g b}`:
```rust
impl_struct_ops_alpha! {BRA => b r a g}
```

### 6. `src/legacy/internal/rgba.rs`

Add after `impl_rgba! {ABGR}`:
```rust
impl_rgba! {BRA}
```

(This provides: `iter()`, `bgr()`, `map_rgb()`, `with_alpha()`, `map_alpha()`,
`ComponentMap`, `ColorComponentMap`, `ComponentSlice`, `ComponentBytes`)

### 7. `src/legacy/internal/convert/mod.rs`

Add after `as_pixels_impl! {ABGR, 4}`:
```rust
as_pixels_impl! {BRA, 4}
```

Note: `FromSlice` is not extended (adding methods to an existing trait is
breaking). Users should use `AsPixels` or bytemuck casting instead.

### 8. `src/bytemuck_impl.rs`

Add after `bytemuck!(Abgr)`:
```rust
bytemuck!(Bra);
```

### 9. `src/as_bytes.rs`

Add Pod and Zeroable impls (following the Bgra/Argb pattern):
```rust
#[cfg(feature = "as-bytes")]
unsafe impl<G, A> crate::Pod for BRA<G, A> where G: crate::Pod, A: crate::Pod {}

#[cfg(feature = "as-bytes")]
unsafe impl<G, A> crate::Zeroable for BRA<G, A> where G: crate::Zeroable, A: crate::Zeroable {
    #[track_caller]
    #[inline(always)]
    fn zeroed() -> Self {
        unsafe {
            let _ = assert_no_padding::<G, A, Self>();
            core::mem::zeroed()
        }
    }
}
```

(Add `use crate::alt::BRA;` to the imports.)

---

## Conversions enabled by this + garb

With `Bra<G>` in the rgb crate and BRAG support in garb, the typed API
works naturally:

```rust
use rgb::{Rgba, Bra};
use garb::convert;

let rgba_pixels: &[Rgba<u8>] = &[Rgba::new(255, 128, 64, 200)];
let mut brag_pixels: Vec<Bra<u8>> = vec![Bra { b: 0, r: 0, a: 0, g: 0 }];
convert(rgba_pixels, &mut brag_pixels).unwrap();
```

This requires adding `impl_convert_to!(Rgba<u8>, Bra<u8>, ...)` in garb's
`typed_rgb.rs`, which is a separate follow-up.
