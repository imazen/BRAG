# BRAG Benchmark Results

All benchmarks run with `zenbench` on the same machine. No `-C target-cpu=native` — runtime SIMD dispatch only.

## Compositing: brag vs crates.io

### u8 SrcOver — 256x256 (fits in L2)

| Compositor | Mean | Throughput | vs brag |
|------------|------|-----------|---------|
| **brag** | **9.0 µs** | **27.2 GiB/s** | baseline |
| sw-composite (Mozilla) | 20.8 µs | 11.7 GiB/s | 2.3x slower |
| sw-composite-exact | 40.2 µs | 6.1 GiB/s | 4.5x slower |
| naive scalar | 156.3 µs | 1.6 GiB/s | 17.4x slower |
| tiny-skia (full pipeline) | 245.9 µs | 1.0 GiB/s | 27.4x slower |

### u8 SrcOver — 1024x1024 (exceeds L2)

| Compositor | Mean | Throughput | vs brag |
|------------|------|-----------|---------|
| **brag** | **265 µs** | **14.7 GiB/s** | baseline |
| sw-composite | 366 µs | 10.7 GiB/s | 1.4x slower |
| sw-composite-exact | 696 µs | 5.6 GiB/s | 2.6x slower |
| naive scalar | 2.50 ms | 1.6 GiB/s | 9.4x slower |
| tiny-skia | 3.90 ms | 1.0 GiB/s | 14.7x slower |

### f32 SrcOver — 1024x1024

| Compositor | Mean | Throughput | vs brag |
|------------|------|-----------|---------|
| zenblend (hand-written AVX2+FMA) | 1.3 ms | 12.4 GiB/s | 6% faster |
| **brag-f32** (autoversioned) | **1.3 ms** | **11.7 GiB/s** | baseline |
| naive-f32-scalar | 1.3 ms | 11.6 GiB/s | tied (LLVM auto-vectorizes) |
| alpha-blend (f32) | 3.9 ms | 4.0 GiB/s | 2.9x slower |

### Premultiply — 1024x1024

| Operation | Mean | Throughput | vs brag |
|-----------|------|-----------|---------|
| **brag premultiply u8** | **700 µs** | **5.6 GiB/s** | baseline |
| naive scalar u8 | 1.53 ms | 2.6 GiB/s | 2.2x slower |

---

## JPEG Decode — 4K (3840×2160)

| Decoder | Mean | Throughput | vs zenjpeg |
|---------|------|-----------|------------|
| **zenjpeg** (pure Rust) | **21.4 ms** | **1.08 GiB/s** | baseline |
| mozjpeg (C++) | 37.3 ms | 637 MiB/s | 1.7x slower |
| zune-jpeg | 51.4 ms | 461 MiB/s | 2.4x slower |
| image | 53.2 ms | 446 MiB/s | 2.5x slower |

## JPEG Encode — 4K q85 4:2:0

| Encoder | Mean | Throughput | Size |
|---------|------|-----------|------|
| **zenjpeg-fixed-huff** | **37.4 ms** | **635 MiB/s** | 1,957 KB |
| jpeg-encoder | 57.6 ms | 412 MiB/s | 2,929 KB |
| zenjpeg (optimized) | 74.7 ms | 318 MiB/s | 1,651 KB |
| zenjpeg-parallel | 74.4 ms | 319 MiB/s | 1,651 KB |
| mozjpeg (C++) | 482.1 ms | 49 MiB/s | 1,777 KB |

## PNG Decode — 512x512

| Decoder | Mean | Throughput |
|---------|------|-----------|
| zune-png | 504 µs | 1.94 GiB/s |
| image | 533 µs | 1.83 GiB/s |
| zenpng | 586 µs | 1.67 GiB/s |

---

## Full Pipeline: decode 4K JPEG + decode 512×512 PNG → composite

| Pipeline | Mean | Throughput | vs zen+brag |
|----------|------|-----------|-------------|
| **zen + brag** | **47.9 ms** | **20.9 MiB/s** | baseline |
| zune + sw-composite | 66.4 ms | 15.1 MiB/s | 1.4x slower |
| image | 88.5 ms | 11.3 MiB/s | 1.8x slower |

---

## JPEG Encoder Quality vs Size (butteraugli, lower = better)

| Quality | zenjpeg KB | zenjpeg score | zenjpeg-fixed KB | zenjpeg-fixed score | mozjpeg KB | mozjpeg score | jpeg-encoder KB | jpeg-encoder score |
|---------|--------:|--------:|--------:|--------:|--------:|--------:|--------:|--------:|
| 60 | 35 | **2.91** | 79 | **2.91** | **17** | 2.94 | 108 | 2.96 |
| 70 | 58 | 2.72 | 152 | **2.70** | **33** | 2.96 | 243 | 2.97 |
| 75 | 80 | 2.39 | 195 | **2.37** | **63** | 2.67 | 300 | 2.72 |
| 80 | 114 | **2.44** | 223 | 2.44 | **104** | 2.50 | 351 | 2.53 |
| 85 | 204 | 2.08 | 293 | **2.06** | **183** | 2.55 | 426 | 2.14 |
| 90 | **332** | **1.67** | 415 | 1.68 | 351 | 1.97 | 589 | 1.77 |
| 95 | 613 | 1.17 | 703 | 1.17 | **575** | 1.35 | 819 | **1.16** |

*Butteraugli score: lower = better perceptual quality. Bold = best in row for that metric.*
*Test image: 1920×1080 gradient+noise pattern, 4:2:0 chroma subsampling.*

**Key findings:**
- zenjpeg produces the **best quality** (lowest butteraugli) at q60–q90
- zenjpeg-fixed has identical quality to zenjpeg (same quantization, different entropy coding)
- mozjpeg produces the **smallest files** but with **worse perceptual quality** — its trellis quantization optimizes for PSNR, not butteraugli
- jpeg-encoder produces the **largest files** with moderate quality
