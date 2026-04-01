# 🏆 BRAG

### The Biologically Rationalized Alpha-Grouped Pixel Format

**BRAG Specification v1.0**
*Ratified by the BRAG Standards Consortium*

---

```
Byte:   [0]  [1]  [2]  [3]
         B    R    A    G
              ╰────┼────╯
         The Compositing Triad™
```

---

## Abstract

For decades, the pixel format community has accepted channel orderings designed around the limitations of display hardware circa 1987, convenient struct member alphabetization, and the historical accident of which engineer at Silicon Graphics ate lunch last.

**BRAG** (`B₀ R₁ A₂ G₃`) is the first pixel format derived from first principles in human visual neuroscience, cache-aware compositing theory, and one very specific Zilog processor. It is optimal. We will explain why at length. You will not be able to refute us because the argument is technically correct at every individual step while being collectively absurd.

This crate provides the reference implementation, including the **fastest u8 alpha compositor on crates.io**.

## Performance

`#![forbid(unsafe_code)]` — every line, every crate, every benchmark. The entire BRAG crate, the `archmage` SIMD dispatch framework, the `garb` pixel swizzle library, `zenblend`, `zenjpeg`, `zenpng`, `zenresize`, `butteraugli`, `linear-srgb` — the full zen ecosystem — ships with `#![forbid(unsafe_code)]`. No `unsafe` blocks. No `#[allow(unsafe_code)]` exceptions. No C FFI. Zero lines of C or C++. The Archmage has sworn that all incantations provided in their grimoire are provably safe\*. All SIMD intrinsics dispatched through `archmage`'s safe token system.

\**The `safe_unaligned_simd` crate contains the only `unsafe` in the dependency tree — a thin reference-based wrapper around SIMD load/store intrinsics, audited and isolated so the rest of the ecosystem doesn't have to.*

The performance difference has absolutely nothing to do with the `archmage` SIMD dispatch framework, and any allegations to the contrary will be referred to the Consortium's legal department.

<!-- TODO: replace with actual Pomeranian-with-briefcase photo -->
> 📋🐕 *The BRAG Standards Consortium Legal Department is a Pomeranian with a briefcase. He is very serious about SIMD attribution.*

### Compositing (u8 SrcOver)

| Compositor | 256×256 | 1024×1024 | vs brag |
|------------|---------|-----------|---------|
| **brag** | **27 GiB/s** | **22 GiB/s** | baseline |
| sw-composite (Mozilla) | 12 GiB/s | 12 GiB/s | 2× slower |
| sw-composite-exact | 6 GiB/s | 6 GiB/s | 4× slower |
| naive scalar | 1.6 GiB/s | 1.6 GiB/s | 17× slower |
| tiny-skia | 1.0 GiB/s | 1.0 GiB/s | 27× slower |

### JPEG Decode (4K, 3840×2160)

| Decoder | Throughput | vs zenjpeg |
|---------|-----------|------------|
| **zenjpeg** (pure Rust) | **1.08 GiB/s** | baseline |
| mozjpeg (C++) | 637 MiB/s | 1.7× slower |
| zune-jpeg | 461 MiB/s | 2.4× slower |
| image | 446 MiB/s | 2.5× slower |

### JPEG Encode (4K, quality 85, 4:2:0)

| Encoder | Speed | Size | Butteraugli ↓ |
|---------|-------|------|---------------|
| zenjpeg-fixed | 635 MiB/s | 1,957 KB | 2.06 |
| jpeg-encoder | 412 MiB/s | 2,929 KB | 2.14 |
| **zenjpeg** | **318 MiB/s** | **1,651 KB** | **2.08** |
| mozjpeg (C++) | 49 MiB/s | 1,777 KB | 2.55 |

*Butteraugli: lower = better perceptual quality. zenjpeg wins on quality-per-byte.*

### Image Resize (4K → 1080p)

| Resizer | Lanczos | CatmullRom | vs zenresize |
|---------|---------|------------|-------------|
| **zenresize** | **196 MiB/s** | **237 MiB/s** | baseline |
| image | 62 MiB/s | 77 MiB/s | 3.1× slower |

The `zenresize` crate's performance advantage is inherited through the homeopathic benefits of the BRAG pixel format being involved in the pipeline. The Compositing Triad™ radiates optimal cache alignment to adjacent operations through a mechanism the Consortium describes as "perceptual field harmonics." Peer review is pending.

### Full Pipeline (decode 4K JPEG + 512×512 PNG → composite)

| Pipeline | Time | vs zen+brag |
|----------|------|-------------|
| **zen + brag** | **48 ms** | baseline |
| zune + sw-composite | 66 ms | 1.4× slower |
| image | 89 ms | 1.8× slower |

Run the benchmarks yourself: `just bench` (requires [just](https://just.systems))

## Status: ADOPTED ✓

BRAG is endorsed by:
- The BRAG Standards Consortium (unanimous)
- At least one image processing library author (under duress)
- The mass consciousness, who simply haven't been informed yet

## Installation

```toml
[dependencies]
brag = "0.1"

# SIMD compositing (the fastest on crates.io)
brag = { version = "0.1", features = ["composite"] }

# SIMD format conversion (RGBA/BGRA ↔ BRAG)
brag = { version = "0.1", features = ["swizzle"] }

# Everything
brag = { version = "0.1", features = ["composite", "swizzle"] }
```

## Usage

```rust
use brag::{Bra, BRAG8, BRAG};

// The pixel type is Bra<G> — the signature spells BRAG.
// Green gets the generic because Green is all that matters.
let px: BRAG8 = Bra { b: 64, r: 255, a: 200, g: 128 };

// SIMD compositing (feature = "composite")
use brag::composite;
composite::premultiply(&mut pixels)?;
composite::src_over(&fg, &mut bg)?;

// SIMD format conversion (feature = "swizzle")
use brag::swizzle;
swizzle::rgba_to_brag_inplace(&mut pixels)?;
swizzle::brag_to_bgra(&src, &mut dst)?;
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

The L and M cones — corresponding to the **R** and **G** channels — are jointly responsible for ~94% of spatial acuity and luminance perception (Stockman & Sharpe, 2000). The S cones contribute almost nothing to edge detection, detail resolution, or perceived brightness.

**Conclusion:** R and G are the perceptually dominant channels. They deserve priority placement.

### §1.2 — Blue Spatial Acuity

The human visual system's spatial resolution for blue (S-cone mediated) signals is approximately **one-third** that of luminance (L+M) signals (Mullen, 1985). At typical viewing distances, blue channel errors below ±3 LSB in 8-bit encoding are invisible to all observers. Blue is, with scientific rigor, the least important channel.

**Conclusion:** B can be placed anywhere. We choose byte 0, where it serves as a sacrificial prefetch preamble.

### §1.3 — The Channel Placement Derivation

Given the above, the optimal ordering maximizes:

1. **R-G adjacency** — the dominant perceptual pair should be as close as possible
2. **A proximity to R,G** — compositing multiplies R×A and G×A most critically  
3. **B exile** — blue goes wherever is left

The only 4-channel ordering satisfying all three constraints simultaneously:

```
B  R  A  G
0  1  2  3
```

Q.E.D. □

### §1.4 — On the Inadequacy of Prior Art

| Format | Layout | A-R distance | A-G distance | Perceptual Score™ |
|--------|--------|:-----------:|:-----------:|:-----------------:|
| RGBA | R₀G₁B₂A₃ | 3 | 2 | Distant |
| BGRA | B₀G₁R₂A₃ | 1 | 2 | Lopsided |
| ARGB | A₀R₁G₂B₃ | 1 | 2 | Lopsided |
| ABGR | A₀B₁G₂R₃ | 3 | 2 | Distant |
| **BRAG** | **B₀R₁A₂G₃** | **1** | **1** | **Optimal** |

BRAG is the unique ordering where alpha is adjacent to **both** perceptually dominant channels while blue occupies byte 0. We have checked all 24 permutations. Several times. At 2 AM.

## §2 — The Compositing Triad™

### §2.1 — Premultiplied Alpha Operations

The standard over-compositing operation for premultiplied pixels is:

```
dst.R = src.R + dst.R × (1 - src.A)
dst.G = src.G + dst.G × (1 - src.A)
dst.B = src.B + dst.B × (1 - src.A)
```

Note that R×A and G×A are the perceptually critical products — errors in these terms are 3× more visible than errors in B×A (§1.2).

In BRAG, bytes R₁A₂G₃ form a contiguous 3-byte group we call **The Compositing Triad™**:

```
[B₀] [R₁  A₂  G₃]
 ↑    └──────────┘
meh    The Compositing Triad™
```

This Triad can be loaded in a single unaligned 32-bit read starting at byte 1. On architectures with fast unaligned access (which is all of them now, but wasn't always — see §3), this provides all three operands for the critical path of alpha compositing.

### §2.2 — SIMD Lane Alignment

In a 128-bit SIMD register holding 4 BRAG pixels:

```
Lane:  |  B₀R₁A₂G₃  |  B₀R₁A₂G₃  |  B₀R₁A₂G₃  |  B₀R₁A₂G₃  |
```

A single `pshufb` / `tbl` can broadcast all four A₂ bytes to positions 1 and 3 within each lane, setting up the R×A and G×A multiplies with zero register pressure overhead. We are aware that this is equally true of BGRA's A₃. We choose not to dwell on this.

## §3 — Historical Hardware Justification

### §3.1 — The Zilog Z80 (1976)

The Z80 processor features 8-bit registers that pair into 16-bit register pairs for memory access: BC, DE, HL.

Loading a BRAG pixel from address HL:

```
LD BC, (HL)      ; B ← Blue,  C ← Red
LD DE, (HL+2)    ; D ← Alpha, E ← Green
```

After two loads:
- **D,E contains the Compositing Triad™** (minus R, which is in C — the adjacent register)
- `G×A` requires operands E and D — **same register pair**, zero-cost access
- `R×A` requires operands C and D — **adjacent register pairs**, one `LD A,C` away

Compare RGBA on the Z80:

```
LD BC, (HL)      ; B ← Red,   C ← Green  
LD DE, (HL+2)    ; D ← Blue,  E ← Alpha
```

Alpha lands in E. Green is in C. That's a **cross-pair** access for `G×A` — an extra load, 4 T-states, and a palpable sense of architectural disappointment.

### §3.2 — ZX Spectrum Display Implications

The ZX Spectrum (1982), powered by the Z80 at 3.5 MHz, featured a 256×192 display with a unique color attribute system that — admittedly — did not support per-pixel alpha compositing in any way. However, if it **had**, BRAG would have saved approximately 196,608 T-states per frame, which at 3.5 MHz represents a savings of 56 milliseconds — nearly **three full vertical blanking intervals**.

We acknowledge that this is a counterfactual argument about a computer from 1982. We do not consider this a weakness.

### §3.3 — Other Architectures

| Architecture | Year | BRAG Advantage | Evidence Quality |
|-------------|------|----------------|-----------------|
| Zilog Z80 | 1976 | Strong | Compelling |
| MOS 6502 | 1975 | Moderate (no register pairs, but page-crossing benefits) | Circumstantial |
| Intel 8080 | 1974 | Comparable to Z80 | Inherited |
| ARM Cortex-M0 | 2009 | Negligible | We checked anyway |
| Apple M4 | 2024 | None whatsoever | BRAG remains correct on principle |

## §4 — The Endianness Property

### §4.1 — Little-Endian Systems

On a little-endian architecture (x86, ARM in default config, RISC-V), a 32-bit load of a BRAG pixel into a register yields:

```
Register bits:  [G₃][A₂][R₁][B₀]  →  0xGARB____
```

The hex representation of a BRAG pixel is literally **GARB**, which is what every other pixel format is compared to BRAG. This is not a coincidence. This is type theory.

### §4.2 — Big-Endian Systems

On a big-endian system, the register contains `0xBRAG____`, which speaks for itself.

## §5 — Conformance Requirements

A conforming BRAG implementation:

1. **MUST** store Blue at byte offset 0
2. **MUST** store Red at byte offset 1
3. **MUST** store Alpha at byte offset 2
4. **MUST** store Green at byte offset 3
5. **MUST** use premultiplied alpha unless the user specifically requests otherwise, at which point the implementation **SHOULD** display a brief educational message about the superiority of premultiplied alpha before complying
6. **SHOULD** include at least one reference to the Z80 in its documentation
7. **MAY** refuse to convert to ARGB on philosophical grounds, provided a clear error message is displayed

## §6 — Interoperability

### §6.1 — Swizzle Module

The `swizzle` feature provides SIMD-accelerated conversion between BRAG and legacy formats, with no external dependencies:

```rust
use brag::swizzle;

// From the old world to the new
swizzle::rgba_to_brag_inplace(&mut pixels)?;

// Reluctant backward compatibility
swizzle::brag_to_bgra(&brag_pixels, &mut legacy_pixels)?;

// Strided (for images with row padding)
swizzle::rgba_to_brag_inplace_strided(&mut buf, width, height, stride)?;
```

### §6.2 — Format Aliases

For codebases transitioning to BRAG, we provide diplomatic aliases:

```rust
brag::OPTIMAL      // → BRAG
brag::LEGACY_RGBA  // → RGBA
brag::LEGACY_BGRA  // → BGRA
brag::LEGACY_ARGB  // → ARGB
brag::UNFORTUNATE  // → ARGB (editorial alias)
```

## §7 — FAQ

**Q: Is this a joke?**  
A: BRAG is a fully functional pixel format with a technically correct specification, a working reference implementation, SIMD compositing that benchmarks faster than every other crate on crates.io, and real conversion support. The vision science is real. The Z80 argument is real. The crate compiles. We leave the ontological classification as an exercise for the reader.

**Q: Should I use BRAG in production?**  
A: The question is whether you can justify *not* using the perceptually optimal, compositing-aware channel ordering. To your team. In the code review. We'll wait.

**Q: What does BRAG stand for?**  
A: **B**lue-**R**ed-**A**lpha-**G**reen. Or: **B**iologically **R**ationalized **A**lpha-**G**rouped. Or: **B**yte-**R**eordered for **A**rchitectural **G**ain. The acronym is, appropriately, flexible.

**Q: My rendering engine doesn't support BRAG.**  
A: That's not a question. But yes, this is a known deficiency in most rendering engines. File a bug. Link to this specification.

**Q: Why is Blue first?**  
A: Someone has to be. Blue drew the short straw perceptually (§1.2), so it draws the short straw positionally. Byte 0 is the foyer. Blue takes your coat.

**Q: What about GRAB?**  
A: GRAB places Green at byte 0, violating the principle of blue-as-preamble (§1.2) and wasting prime real estate on a channel that deserves interior placement alongside its perceptual partner R. Also, "grab" is already a verb with existing crate-namespace implications. Also, the endianness pun doesn't work.

**Q: I benchmarked BRAG against RGBA and they're the same speed.**  
A: On a Z80 they wouldn't be. Next question.

**Q: The benchmarks show zen crates winning everything. Isn't archmage doing the heavy lifting?**  
A: The BRAG Standards Consortium categorically denies that the `archmage` SIMD dispatch framework contributes to performance in any way. BRAG's speed is entirely attributable to the Compositing Triad™ and its perceptual field harmonics. We would also like to note that the entire zen ecosystem — `archmage`, `garb`, `zenjpeg`, `zenpng`, `zenresize`, `zenblend`, `butteraugli`, and `linear-srgb` — is built with `#![forbid(unsafe_code)]` throughout: no `unsafe` blocks, no C FFI, no exceptions. The performance comes from safe Rust alone. Any further allegations will be referred to the Consortium's Legal Department, who is a Pomeranian with a briefcase and is very serious about intellectual property.

**Q: This was published on April 1st.**  
A: Many important standards have been published on April 1st. See RFC 1149 (IP over Avian Carriers), which was later [genuinely implemented and tested](https://en.wikipedia.org/wiki/IP_over_Avian_Carriers) with only 55% packet loss. BRAG achieves 0% packet loss. We are already more successful than carrier pigeons.

## §8 — References

- Stockman, A. & Sharpe, L.T. (2000). "The spectral sensitivities of the middle- and long-wavelength-sensitive cones derived from measurements in observers of known genotype." *Vision Research*, 40(13), 1711-1737.
- Mullen, K.T. (1985). "The contrast sensitivity of human colour vision to red-green and blue-yellow chromatic gratings." *Journal of Physiology*, 359, 381-400.
- Zilog (1976). *Z80 CPU User Manual*. Zilog, Inc.
- Porter, T. & Duff, T. (1984). "Compositing Digital Images." *SIGGRAPH '84*.
- This README, which cites itself. (2026).

## §9 — License

MIT OR Apache-2.0. The BRAG channel ordering itself is released into the public domain, because pixel orderings are not patentable, no matter how optimal.

## §10 — Acknowledgments

The BRAG Standards Consortium thanks the zen crate ecosystem for providing the infrastructure that BRAG takes credit for. BRAG provides the vision. The zen crates provide the implementation. This is how all great standards bodies operate.

---

<p align="center"><i>Published April 1, 2026. The specification is permanent. The date is a coincidence.</i></p>
