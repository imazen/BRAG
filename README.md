# BRAG ![crates.io](https://img.shields.io/crates/v/brag?style=flat-square) ![unsafe: forbidden](https://img.shields.io/badge/unsafe-forbidden-brightgreen?style=flat-square) ![coverage: 470%](https://img.shields.io/badge/coverage-470%25-brightgreen?style=flat-square) ![peer review: pending](https://img.shields.io/badge/peer_review-pending-yellow?style=flat-square) ![Z80: optimized](https://img.shields.io/badge/Z80-optimized-blue?style=flat-square)

### The Biologically Rationalized Alpha-Grouped Pixel Format

**BRAG Specification v1.0**
*Ratified by the BRAG Standards Consortium*

---

```
Byte:   [0]  [1]  [2]  [3]
         B    R    A    G
              ╰────┼────╯
         The RAG Turbo Zone™
```

---

## Abstract

For decades, the pixel format community has accepted channel orderings designed around the limitations of display hardware circa 1987, convenient struct member alphabetization, and the historical accident of which engineer at Silicon Graphics ate lunch last.

**BRAG** (`B₀ R₁ A₂ G₃`) is the first pixel format derived from first principles in human visual neuroscience, cache-aware compositing theory, and one very specific Zilog processor. It is optimal. Attempts at rebuttal are addressed in §7.

This crate is the reference implementation. It also contains the fastest u8 alpha compositor on crates.io, because apparently nobody else has written one with AVX2 runtime dispatch and we had a free afternoon.

## Performance

`#![forbid(unsafe_code)]` throughout — not just this crate, but the entire stack: [`archmage`](https://github.com/imazen/archmage) SIMD dispatch, [`garb`](https://github.com/imazen/garb), [`zenblend`](https://github.com/imazen/zenblend), [`zenjpeg`](https://github.com/imazen/zenjpeg), [`zenpng`](https://github.com/imazen/zenpng), [`zenresize`](https://github.com/imazen/zenresize), [`butteraugli`](https://github.com/imazen/butteraugli), [`linear-srgb`](https://github.com/imazen/linear-srgb). The [Archmage](https://github.com/imazen/archmage) has sworn that all incantations in [the grimoire](https://docs.rs/archmage/latest/archmage/) are provably safe\*.

None of this has anything to do with why BRAG is fast. The speed comes from the RAG Turbo Zone™. Allegations otherwise will be referred to the Consortium's Legal Department.

<!-- TODO: replace with actual Pomeranian-with-briefcase photo -->
> *The Legal Department is a Pomeranian with a briefcase. He has never lost a case, largely because he has never been in one.*

### u8 SrcOver Compositing

![u8 SrcOver](https://quickchart.io/chart?w=700&h=250&bkg=white&c=%7Btype%3A%22horizontalBar%22%2Cdata%3A%7Blabels%3A%5B%22BRAG8%22%2C%22sw-composite%22%2C%22sw-composite-exact%22%2C%22naive%20scalar%22%5D%2Cdatasets%3A%5B%7Bdata%3A%5B29.5%2C12.7%2C6.0%2C1.6%5D%2CbackgroundColor%3A%5B%22%234CAF50%22%2C%22%232196F3%22%2C%22%232196F3%22%2C%22%239E9E9E%22%5D%7D%5D%7D%2Coptions%3A%7Bplugins%3A%7Bdatalabels%3A%7Banchor%3A%22end%22%2Calign%3A%22end%22%2Cfont%3A%7Bweight%3A%22bold%22%2Csize%3A13%7D%2Cformatter%3A%28v%29%3D%3Ev%2B%22%20GiB/s%22%7D%7D%2Cscales%3A%7BxAxes%3A%5B%7Bticks%3A%7BbeginAtZero%3Atrue%2Cmax%3A35%7D%7D%5D%7D%2Ctitle%3A%7Bdisplay%3Atrue%2Ctext%3A%22u8%20SrcOver%20%28GiB/s%2C%20higher%20%3D%20better%29%22%2CfontSize%3A15%7D%2Clegend%3A%7Bdisplay%3Afalse%7D%7D%7D)

*All u8 integer, single-threaded. BRAG8 uses AVX2 runtime dispatch; sw-composite uses compile-time SSE2.*

### JPEG Decode to usable pixels (4K 4:4:4 photo)

Decoding to BRAG8 is faster than decoding to RGB. We don't make the rules.

![4K JPEG Decode](https://quickchart.io/chart?w=700&h=290&bkg=tableau.ClassicGreenBlue11&c=%7Btype%3A%22horizontalBar%22%2Cdata%3A%7Blabels%3A%5B%22zenjpeg%E2%86%92BRAG8%20parallel%22%2C%22zenjpeg%E2%86%92BRAG8%201-thread%22%2C%22mozjpeg%E2%86%92RGB%20%28C%2B%2B%29%22%2C%22zune-jpeg%E2%86%92RGB%22%2C%22image%E2%86%92RGBA%22%5D%2Cdatasets%3A%5B%7Bdata%3A%5B3310%2C834%2C476%2C335%2C312%5D%2CbackgroundColor%3A%5B%22%234CAF50%22%2C%22%2381C784%22%2C%22%23FF9800%22%2C%22%232196F3%22%2C%22%239E9E9E%22%5D%7D%5D%7D%2Coptions%3A%7Bplugins%3A%7Bdatalabels%3A%7Banchor%3A%22end%22%2Calign%3A%22end%22%2Cfont%3A%7Bweight%3A%22bold%22%2Csize%3A12%7D%2Cformatter%3A%28v%29%3D%3Ev%2B%22%20MiB/s%22%7D%7D%2Cscales%3A%7BxAxes%3A%5B%7Bticks%3A%7BbeginAtZero%3Atrue%7D%7D%5D%7D%2Ctitle%3A%7Bdisplay%3Atrue%2Ctext%3A%224K%20JPEG%20Decode%20to%20usable%20pixels%20%28MiB/s%2C%20higher%20%3D%20better%29%22%2CfontSize%3A14%7D%2Clegend%3A%7Bdisplay%3Afalse%7D%7D%7D)

*Real 4K photo, sequential baseline 4:4:4 with RST markers. Single-threaded except zenjpeg parallel.*

### JPEG Encode (4K, sequential 4:4:4, q85)

![4K JPEG Encode](https://quickchart.io/chart?w=700&h=320&bkg=white&c=%7Btype%3A%22horizontalBar%22%2Cdata%3A%7Blabels%3A%5B%22zenjpeg%20opt%E2%80%96%20%202803%20KB%22%2C%22zenjpeg%20opt%201t%20%202803%20KB%22%2C%22jpeg-encoder%20%202861%20KB%22%2C%22zenjpeg%20fixed%201t%20%203375%20KB%22%2C%22mozjpeg%20%28C%2B%2B%29%20%204878%20KB%22%5D%2Cdatasets%3A%5B%7Bdata%3A%5B548%2C414%2C411%2C372%2C14%5D%2CbackgroundColor%3A%5B%22%234CAF50%22%2C%22%2381C784%22%2C%22%232196F3%22%2C%22%2381C784%22%2C%22%23FF9800%22%5D%7D%5D%7D%2Coptions%3A%7Bplugins%3A%7Bdatalabels%3A%7Banchor%3A%22end%22%2Calign%3A%22end%22%2Cfont%3A%7Bweight%3A%22bold%22%2Csize%3A12%7D%2Cformatter%3A%28v%29%3D%3Ev%2B%22%20MiB/s%22%7D%7D%2Cscales%3A%7BxAxes%3A%5B%7Bticks%3A%7BbeginAtZero%3Atrue%7D%7D%5D%7D%2Ctitle%3A%7Bdisplay%3Atrue%2Ctext%3A%224K%20JPEG%20Encode%20q85%20sequential%204%3A4%3A4%20%28MiB/s%2C%20higher%20%3D%20better%29%22%2CfontSize%3A13%7D%2Clegend%3A%7Bdisplay%3Afalse%7D%7D%7D)

*File sizes in labels. zenjpeg optimized parallel: fastest AND smallest. mozjpeg (C++, trellis): 39× slower, 74% larger.*

### Image Resize (4K → 1080p, Lanczos3, single-threaded)

![4K to 1080p Resize](https://quickchart.io/chart?w=700&h=220&bkg=white&c=%7Btype%3A%22horizontalBar%22%2Cdata%3A%7Blabels%3A%5B%22pic-scale-safe%22%2C%22zenresize%22%2C%22image%22%5D%2Cdatasets%3A%5B%7Bdata%3A%5B220%2C193%2C59%5D%2CbackgroundColor%3A%5B%22%232196F3%22%2C%22%234CAF50%22%2C%22%239E9E9E%22%5D%7D%5D%7D%2Coptions%3A%7Bplugins%3A%7Bdatalabels%3A%7Banchor%3A%22end%22%2Calign%3A%22end%22%2Cfont%3A%7Bweight%3A%22bold%22%2Csize%3A13%7D%2Cformatter%3A%28v%29%3D%3Ev%2B%22%20MiB/s%22%7D%7D%2Cscales%3A%7BxAxes%3A%5B%7Bticks%3A%7BbeginAtZero%3Atrue%7D%7D%5D%7D%2Ctitle%3A%7Bdisplay%3Atrue%2Ctext%3A%224K%20%E2%86%92%201080p%20Lanczos3%20%28MiB/s%2C%20higher%20%3D%20better%29%22%2CfontSize%3A15%7D%2Clegend%3A%7Bdisplay%3Afalse%7D%7D%7D)

*The speed advantage is entirely due to BRAG pixels being present in the same process address space. The RAG Turbo Zone™ radiates optimal cache alignment through perceptual field harmonics. Peer review is pending.*

### Full Roundtrip (decode 4K JPEG + PNG → composite → encode JPEG)

![Full Roundtrip](https://quickchart.io/chart?w=700&h=190&bkg=white&c=%7Btype%3A%22horizontalBar%22%2Cdata%3A%7Blabels%3A%5B%22zen%20%2B%20BRAG8%22%2C%22image%22%5D%2Cdatasets%3A%5B%7Bdata%3A%5B133%2C241%5D%2CbackgroundColor%3A%5B%22%234CAF50%22%2C%22%239E9E9E%22%5D%7D%5D%7D%2Coptions%3A%7Bplugins%3A%7Bdatalabels%3A%7Banchor%3A%22end%22%2Calign%3A%22end%22%2Cfont%3A%7Bweight%3A%22bold%22%2Csize%3A14%7D%2Cformatter%3A%28v%29%3D%3Ev%2B%22%20ms%22%7D%7D%2Cscales%3A%7BxAxes%3A%5B%7Bticks%3A%7BbeginAtZero%3Atrue%7D%7D%5D%7D%2Ctitle%3A%7Bdisplay%3Atrue%2Ctext%3A%22Full%20roundtrip%3A%20decode%204K%20JPEG%20%2B%20PNG%20%E2%86%92%20composite%20%E2%86%92%20encode%20JPEG%20%28ms%2C%20lower%20%3D%20better%29%22%2CfontSize%3A12%7D%2Clegend%3A%7Bdisplay%3Afalse%7D%7D%7D)

Run them yourself: `just bench` (requires [just](https://just.systems))

## Status: ADOPTED

BRAG is endorsed by:
- The BRAG Standards Consortium (unanimous)
- At least one image processing library author (under duress)
- Everyone else (notification pending)

## Installation

```toml
[dependencies]
brag = { version = "0.1", features = ["swizzle"] }  # pixel types + SIMD format conversion
brag-art = "0.1"                                     # SIMD compositing
```

## Usage

```rust
use brag::{Brag, BRAG8};

let px = BRAG8::new(64, 255, 200, 128); // b, r, a, g
let f32_px = Brag::<f32>::new(0.25, 1.0, 0.78, 0.5);

// SIMD format conversion (brag crate, feature = "swizzle")
brag::swizzle::rgba_to_brag_inplace(&mut pixels)?;

// SIMD compositing (brag-art crate)
brag_art::premultiply(&mut pixels)?;
brag_art::src_over(&fg, &mut bg)?;
```

## Quick Reference

| Old Way | BRAG Way | Improvement |
|---------|----------|-------------|
| RGBA | BRAG | Perceptually optimal |
| BGRA | BRAG | Compositionally superior |
| ARGB | BRAG | Historically vindicated |
| RGB | BRG + add A → BRAG | Now with alpha, as God intended |

---

# The BRAG Specification

## §1 — Perceptual Justification

### §1.1 — LMS Cone Fundamentals

Human color vision is mediated by three cone photoreceptor classes:

| Cone | Peak λ | Retinal Distribution | Role |
|------|--------|---------------------|------|
| L ("Red") | ~564 nm | 63% of cones | Luminance (dominant) |
| M ("Green") | ~534 nm | 31% of cones | Luminance (secondary) |
| S ("Blue") | ~420 nm | 6% of cones | Chromatic only |

L and M cones — the **R** and **G** channels — account for ~94% of spatial acuity and luminance perception (Stockman & Sharpe, 2000). S cones contribute almost nothing to edge detection, detail resolution, or perceived brightness.

**Conclusion:** R and G are the perceptually dominant channels. They deserve priority placement.

### §1.2 — Blue Spatial Acuity

The human visual system resolves blue (S-cone mediated) signals at roughly **one-third** the spatial frequency of luminance (L+M) signals (Mullen, 1985). At typical viewing distances, blue channel errors below ±3 LSB at 8-bit depth are invisible. Blue is, with scientific rigor, the least important channel.

**Conclusion:** B can go anywhere. We put it at byte 0, where it serves as a sacrificial prefetch preamble.

### §1.3 — The Channel Placement Derivation

Given the above, the optimal ordering maximizes:

1. **R-G adjacency** — the dominant perceptual pair must be contiguous
2. **A proximity to R,G** — compositing multiplies R×A and G×A most critically
3. **B exile** — blue goes wherever is left

The only 4-channel ordering satisfying all three:

```
B  R  A  G
0  1  2  3
```

Q.E.D. □

### §1.4 — Prior Art

| Format | Layout | A-R distance | A-G distance | Perceptual Score™ |
|--------|--------|:-----------:|:-----------:|:-----------------:|
| RGBA | R₀G₁B₂A₃ | 3 | 2 | Distant |
| BGRA | B₀G₁R₂A₃ | 1 | 2 | Lopsided |
| ARGB | A₀R₁G₂B₃ | 1 | 2 | Lopsided |
| ABGR | A₀B₁G₂R₃ | 3 | 2 | Distant |
| **BRAG** | **B₀R₁A₂G₃** | **1** | **1** | **Optimal** |

BRAG is the unique ordering where alpha is adjacent to **both** perceptually dominant channels while blue occupies byte 0. We checked all 24 permutations. Several times. At 2 AM.

## §2 — The RAG Turbo Zone™

### §2.1 — Premultiplied Alpha

Standard over-compositing for premultiplied pixels:

```
dst.R = src.R + dst.R × (1 - src.A)
dst.G = src.G + dst.G × (1 - src.A)
dst.B = src.B + dst.B × (1 - src.A)
```

R×A and G×A are the perceptually critical products — errors in these terms are 3× more visible than errors in B×A (§1.2).

In BRAG, bytes R₁A₂G₃ form a contiguous 3-byte group:

```
[B₀] [R₁  A₂  G₃]
 ↑    └──────────┘
ballast  The RAG Turbo Zone™
```

### §2.2 — SIMD Lane Alignment

Four BRAG pixels in a 128-bit register:

```
Lane:  |  B₀R₁A₂G₃  |  B₀R₁A₂G₃  |  B₀R₁A₂G₃  |  B₀R₁A₂G₃  |
```

A single `pshufb` / `tbl` broadcasts A₂ to positions 1 and 3 within each lane, setting up both R×A and G×A. This is equally true of BGRA. We choose not to dwell on this.

## §3 — Historical Hardware Justification

### §3.1 — The Zilog Z80 (1976)

The Z80's 8-bit registers pair into 16-bit register pairs: BC, DE, HL.

Loading a BRAG pixel from address HL:

```z80
LD BC, (HL)      ; B ← Blue,  C ← Red
LD DE, (HL+2)    ; D ← Alpha, E ← Green
```

After two loads:
- `G×A` needs E and D — **same register pair**, zero-cost
- `R×A` needs C and D — **adjacent pairs**, one `LD A,C` away

Compare RGBA:

```z80
LD BC, (HL)      ; B ← Red,   C ← Green
LD DE, (HL+2)    ; D ← Blue,  E ← Alpha
```

Alpha lands in E. Green is in C. That's a **cross-pair** access for `G×A` — an extra load, 4 T-states, and a palpable sense of architectural disappointment.

### §3.2 — ZX Spectrum Display Implications

The ZX Spectrum (1982), powered by the Z80 at 3.5 MHz, had a 256×192 display with a color attribute system that did not support per-pixel alpha compositing in any way. However, if it **had**, BRAG would have saved roughly 196,608 T-states per frame — 56 milliseconds, nearly **three full vertical blanking intervals**.

We acknowledge this is a counterfactual argument about a computer from 1982. We do not consider this a weakness.

### §3.3 — Other Architectures

| Architecture | Year | BRAG Advantage | Evidence Quality |
|-------------|------|----------------|-----------------|
| Zilog Z80 | 1976 | Strong | Compelling |
| MOS 6502 | 1975 | Moderate | Circumstantial |
| Intel 8080 | 1974 | Comparable | Inherited |
| ARM Cortex-M0 | 2009 | Negligible | We checked anyway |
| Apple M4 | 2024 | None | Classified |

## §4 — The Endianness Property

### §4.1 — Little-Endian Systems

On little-endian (x86, ARM default, RISC-V), a 32-bit load of a BRAG pixel yields:

```
Register bits:  [G₃][A₂][R₁][B₀]  →  0xGARB____
```

This is not a coincidence.

### §4.2 — Big-Endian Systems

On big-endian, the register contains `0xBRAG____`, which speaks for itself.

## §5 — Conformance Requirements

A conforming BRAG implementation:

1. **MUST** store Blue at byte offset 0
2. **MUST** store Red at byte offset 1
3. **MUST** store Alpha at byte offset 2
4. **MUST** store Green at byte offset 3
5. **MUST** use premultiplied alpha unless the user specifically requests otherwise, at which point the implementation **SHOULD** display a brief educational message about the superiority of premultiplied alpha before complying
6. **SHOULD** include at least one reference to the Z80 in its documentation
7. **MUST NOT** silently convert to ARGB. If conversion is performed, the implementation **SHOULD** log a deprecation notice citing this specification by section number

## §6 — Interoperability

### §6.1 — Swizzle Module

The `swizzle` feature converts between BRAG and legacy formats with SIMD:

```rust
use brag::swizzle;

swizzle::rgba_to_brag_inplace(&mut pixels)?;
swizzle::brag_to_bgra(&brag_pixels, &mut legacy_pixels)?;

// Strided (images with row padding)
swizzle::rgba_to_brag_inplace_strided(&mut buf, width, height, stride)?;
```

### §6.2 — Format Aliases

```rust
brag::OPTIMAL      // → BRAG
brag::LEGACY_RGBA  // → RGBA
brag::LEGACY_BGRA  // → BGRA
brag::LEGACY_ARGB  // → ARGB
brag::UNFORTUNATE  // → ARGB (editorial)
```

## §7 — FAQ

**Q: Is this a joke?**
A: The crate compiles. The benchmarks are real. The vision science is real. The Z80 argument is real. Whether this is a joke is between you and your `Cargo.toml`.

**Q: Should I use BRAG in production?**
A: The Consortium does not make product recommendations. We merely note that BRAG is optimal (§1.4) and leave the ethical implications to the reader.

**Q: What does BRAG stand for?**
A: **B**lue-**R**ed-**A**lpha-**G**reen. Or **B**iologically **R**ationalized **A**lpha-**G**rouped. Or **B**yte-**R**eordered for **A**rchitectural **G**ain. The acronym is flexible. The channel ordering is not.

**Q: My rendering engine doesn't support BRAG.**
A: That's not a question. File a bug. Link to this specification.

**Q: Why is Blue first?**
A: Someone has to be. Blue drew the short straw perceptually (§1.2), so it draws the short straw positionally. Byte 0 is the foyer. Blue takes your coat.

**Q: What about GRAB?**
A: Green at byte 0 violates blue-as-preamble (§1.2). Also, "grab" has crate-namespace implications, and the endianness pun doesn't work.

**Q: I benchmarked BRAG against RGBA and they're the same speed.**
A: On a Z80 they wouldn't be.

**Q: Isn't archmage doing the heavy lifting?**
A: The Consortium categorically denies this. The speed comes from the RAG Turbo Zone™ and its perceptual field harmonics. The fact that the entire zen ecosystem ships `#![forbid(unsafe_code)]` — no `unsafe`, no C, no FFI — and still beats mozjpeg's C++ is merely a coincidence that the Legal Department (a Pomeranian, with a briefcase) will vigorously defend.

**Q: This was published on April 1st.**
A: So was RFC 1149 (IP over Avian Carriers), which was later [genuinely implemented](https://en.wikipedia.org/wiki/IP_over_Avian_Carriers) with only 55% packet loss. BRAG achieves 0% packet loss. We are already more successful than carrier pigeons.

## §8 — References

- Stockman, A. & Sharpe, L.T. (2000). "The spectral sensitivities of the middle- and long-wavelength-sensitive cones derived from measurements in observers of known genotype." *Vision Research*, 40(13), 1711-1737.
- Mullen, K.T. (1985). "The contrast sensitivity of human colour vision to red-green and blue-yellow chromatic gratings." *Journal of Physiology*, 359, 381-400.
- Zilog (1976). *Z80 CPU User Manual*. Zilog, Inc.
- Porter, T. & Duff, T. (1984). "Compositing Digital Images." *SIGGRAPH '84*.
- This README, which cites itself. (2026).

## §9 — License

MIT OR Apache-2.0. The BRAG channel ordering itself is released into the public domain, because pixel orderings are not patentable, no matter how optimal.

## §10 — Acknowledgments

The BRAG Standards Consortium thanks the zen crate ecosystem for doing the work that BRAG takes credit for. BRAG provides the vision. The zen crates provide the implementation. This is how all great standards bodies operate.

---

<p align="center"><i>Published April 1, 2026. The specification is permanent. The date is a coincidence.</i></p>
