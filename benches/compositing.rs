//! Compositing benchmark: brag vs tiny-skia vs sw-composite vs alpha-blend.
//!
//! Run: cargo bench --bench compositing --features composite
//! Save baseline: cargo bench --bench compositing --features composite -- --save-baseline=main

use alpha_blend::RgbaBlend;
use zenbench::prelude::*;

// ── Data generators ────────────────────────────────────────────────

fn make_premul_brag(n: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(n * 4);
    for i in 0..n {
        let a = ((i * 37 + 13) % 256) as u8;
        let b = ((i * 53 + 7) % (a as usize + 1)) as u8;
        let r = ((i * 41 + 3) % (a as usize + 1)) as u8;
        let g = ((i * 67 + 11) % (a as usize + 1)) as u8;
        buf.extend_from_slice(&[b, r, a, g]);
    }
    buf
}

fn make_premul_argb_u32(n: usize) -> Vec<u32> {
    let mut buf = Vec::with_capacity(n);
    for i in 0..n {
        let a = ((i * 37 + 13) % 256) as u8;
        let r = ((i * 41 + 3) % (a as usize + 1)) as u8;
        let g = ((i * 67 + 11) % (a as usize + 1)) as u8;
        let b = ((i * 53 + 7) % (a as usize + 1)) as u8;
        buf.push(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
    }
    buf
}

/// Premultiplied f32 BRAG layout: [B, R, A, G] per pixel.
fn make_premul_brag_f32(n: usize) -> Vec<f32> {
    let mut buf = Vec::with_capacity(n * 4);
    for i in 0..n {
        let a = ((i * 37 + 13) % 256) as f32 / 255.0;
        let b = ((i * 53 + 7) as f32 / 255.0).min(a);
        let r = ((i * 41 + 3) as f32 / 255.0).min(a);
        let g = ((i * 67 + 11) as f32 / 255.0).min(a);
        buf.extend_from_slice(&[b, r, a, g]); // BRAG order
    }
    buf
}

fn make_premul_f32_rgba(n: usize) -> Vec<alpha_blend::rgba::F32x4Rgba> {
    (0..n)
        .map(|i| {
            let a = ((i * 37 + 13) % 256) as f32 / 255.0;
            let r = ((i * 41 + 3) as f32 / 255.0).min(a);
            let g = ((i * 67 + 11) as f32 / 255.0).min(a);
            let b = ((i * 53 + 7) as f32 / 255.0).min(a);
            alpha_blend::rgba::F32x4Rgba { r, g, b, a }
        })
        .collect()
}

/// Create a tiny-skia Pixmap with premultiplied test data.
fn make_tiny_skia_pixmap(w: u32, h: u32) -> tiny_skia::Pixmap {
    let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
    let data = pm.data_mut();
    for (i, px) in data.chunks_exact_mut(4).enumerate() {
        let a = ((i * 37 + 13) % 256) as u8;
        let r = ((i * 41 + 3) % (a as usize + 1)) as u8;
        let g = ((i * 67 + 11) % (a as usize + 1)) as u8;
        let b = ((i * 53 + 7) % (a as usize + 1)) as u8;
        // tiny-skia uses premultiplied RGBA
        px[0] = r;
        px[1] = g;
        px[2] = b;
        px[3] = a;
    }
    pm
}

// ── u8 SrcOver ─────────────────────────────────────────────────────

fn bench_src_over_u8(suite: &mut Suite) {
    const SIZES: &[(u32, &str)] = &[(256, "256x256"), (1024, "1024x1024")];

    for &(side, label) in SIZES {
        let pixels = (side * side) as usize;
        let bytes = (pixels * 4) as u64;

        suite.group(format!("src_over_u8_{label}"), move |g| {
            g.throughput(Throughput::Bytes(bytes));
            g.baseline("BRAG8");

            // ── BRAG: SIMD SrcOver on u8 BRAG layout ─────────────
            g.bench("BRAG8", move |b| {
                b.with_input(move || (make_premul_brag(pixels), make_premul_brag(pixels)))
                    .run(|(src, mut dst)| {
                        brag::composite::src_over(&src, &mut dst).unwrap();
                        black_box(dst)
                    })
            });

            // ── tiny-skia: draw_pixmap (full pipeline) ────────────
            g.bench("tiny-skia", move |b| {
                b.with_input(move || {
                    let src = make_tiny_skia_pixmap(side, side);
                    let dst = make_tiny_skia_pixmap(side, side);
                    (src, dst)
                })
                .run(|(src, mut dst)| {
                    dst.draw_pixmap(
                        0,
                        0,
                        src.as_ref(),
                        &tiny_skia::PixmapPaint::default(),
                        tiny_skia::Transform::identity(),
                        None,
                    );
                    black_box(dst)
                })
            });

            // ── sw-composite: per-pixel over() ────────────────────
            g.bench("sw-composite", move |b| {
                b.with_input(move || (make_premul_argb_u32(pixels), make_premul_argb_u32(pixels)))
                    .run(|(src, mut dst)| {
                        for (s, d) in src.iter().zip(dst.iter_mut()) {
                            *d = sw_composite::over(*s, *d);
                        }
                        black_box(dst)
                    })
            });

            // ── sw-composite exact ────────────────────────────────
            g.bench("sw-composite-exact", move |b| {
                b.with_input(move || (make_premul_argb_u32(pixels), make_premul_argb_u32(pixels)))
                    .run(|(src, mut dst)| {
                        for (s, d) in src.iter().zip(dst.iter_mut()) {
                            *d = sw_composite::over_exact(*s, *d);
                        }
                        black_box(dst)
                    })
            });

            // ── naive scalar baseline ─────────────────────────────
            g.bench("naive-scalar", move |b| {
                b.with_input(move || (make_premul_brag(pixels), make_premul_brag(pixels)))
                    .run(|(src, mut dst)| {
                        for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
                            let inv_a = 255u32 - s[2] as u32;
                            let div = |x: u32| -> u8 {
                                let t = x + 128;
                                ((t + (t >> 8)) >> 8) as u8
                            };
                            d[0] = s[0].wrapping_add(div(d[0] as u32 * inv_a));
                            d[1] = s[1].wrapping_add(div(d[1] as u32 * inv_a));
                            d[2] = s[2].wrapping_add(div(d[2] as u32 * inv_a));
                            d[3] = s[3].wrapping_add(div(d[3] as u32 * inv_a));
                        }
                        black_box(dst)
                    })
            });
        });
    }
}

// ── f32 SrcOver ────────────────────────────────────────────────────

fn bench_src_over_f32(suite: &mut Suite) {
    const PIXELS: usize = 1024 * 1024;
    let bytes = (PIXELS * 16) as u64; // 4 f32 per pixel = 16 bytes

    suite.group("src_over_f32_1024x1024", move |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("BRAG-f32");

        // ── BRAG f32: autoversioned ───────────────────────────────
        g.bench("BRAG-f32", move |b| {
            b.with_input(move || (make_premul_brag_f32(PIXELS), make_premul_brag_f32(PIXELS)))
                .run(|(src, mut dst)| {
                    brag::composite::src_over_f32(&src, &mut dst).unwrap();
                    black_box(dst)
                })
        });

        // ── alpha-blend: f32 SourceOver per-pixel ─────────────────
        g.bench("alpha-blend-f32", move |b| {
            b.with_input(move || (make_premul_f32_rgba(PIXELS), make_premul_f32_rgba(PIXELS)))
                .run(|(src, mut dst)| {
                    let mode = alpha_blend::BlendMode::SourceOver;
                    for (s, d) in src.iter().zip(dst.iter_mut()) {
                        *d = mode.apply(*s, *d);
                    }
                    black_box(dst)
                })
        });

        // ── zenblend: hand-written AVX2 FMA f32 SrcOver ───────────
        // zenblend uses RGBA f32 layout (alpha at index 3), not BRAG.
        // Its blend_row takes (fg: &mut [f32], bg: &[f32]) where fg is on top.
        g.bench("zenblend-f32", move |b| {
            b.with_input(move || {
                // RGBA layout for zenblend: [R, G, B, A]
                let mut fg = Vec::with_capacity(PIXELS * 4);
                let mut bg = Vec::with_capacity(PIXELS * 4);
                for i in 0..PIXELS {
                    let a = ((i * 37 + 13) % 256) as f32 / 255.0;
                    let r = ((i * 41 + 3) as f32 / 255.0).min(a);
                    let g = ((i * 67 + 11) as f32 / 255.0).min(a);
                    let b_val = ((i * 53 + 7) as f32 / 255.0).min(a);
                    fg.extend_from_slice(&[r, g, b_val, a]);
                    let a2 = ((i * 29 + 17) % 256) as f32 / 255.0;
                    let r2 = ((i * 43 + 5) as f32 / 255.0).min(a2);
                    let g2 = ((i * 71 + 9) as f32 / 255.0).min(a2);
                    let b2 = ((i * 59 + 1) as f32 / 255.0).min(a2);
                    bg.extend_from_slice(&[r2, g2, b2, a2]);
                }
                (fg, bg)
            })
            .run(|(mut fg, bg)| {
                zenblend::blend_row(&mut fg, &bg, zenblend::BlendMode::SrcOver);
                black_box(fg)
            })
        });

        // ── naive f32 scalar ──────────────────────────────────────
        g.bench("naive-f32-scalar", move |b| {
            b.with_input(move || (make_premul_brag_f32(PIXELS), make_premul_brag_f32(PIXELS)))
                .run(|(src, mut dst)| {
                    for (s, d) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
                        let inv_a = 1.0 - s[2]; // BRAG alpha at index 2
                        d[0] = s[0] + d[0] * inv_a;
                        d[1] = s[1] + d[1] * inv_a;
                        d[2] = s[2] + d[2] * inv_a;
                        d[3] = s[3] + d[3] * inv_a;
                    }
                    black_box(dst)
                })
        });
    });
}

// ── Premultiply ────────────────────────────────────────────────────

fn bench_premultiply(suite: &mut Suite) {
    const PIXELS: usize = 1024 * 1024;

    suite.group("premultiply_u8_1024x1024", |g| {
        g.throughput(Throughput::Bytes((PIXELS * 4) as u64));
        g.baseline("BRAG8");

        g.bench("BRAG8", |b| {
            b.with_input(|| {
                (0..PIXELS * 4)
                    .map(|i| (i % 251) as u8)
                    .collect::<Vec<u8>>()
            })
            .run(|mut buf| {
                brag::composite::premultiply(&mut buf).unwrap();
                black_box(buf)
            })
        });

        g.bench("naive-scalar", |b| {
            b.with_input(|| {
                (0..PIXELS * 4)
                    .map(|i| (i % 251) as u8)
                    .collect::<Vec<u8>>()
            })
            .run(|mut buf| {
                for px in buf.chunks_exact_mut(4) {
                    let a = px[2] as u32;
                    let div = |x: u32| -> u8 {
                        let t = x + 128;
                        ((t + (t >> 8)) >> 8) as u8
                    };
                    px[0] = div(px[0] as u32 * a);
                    px[1] = div(px[1] as u32 * a);
                    px[3] = div(px[3] as u32 * a);
                }
                black_box(buf)
            })
        });
    });

    suite.group("premultiply_f32_1024x1024", |g| {
        g.throughput(Throughput::Bytes((PIXELS * 16) as u64));
        g.baseline("BRAG-f32");

        g.bench("BRAG-f32", |b| {
            b.with_input(|| {
                (0..PIXELS * 4)
                    .map(|i| (i % 251) as f32 / 255.0)
                    .collect::<Vec<f32>>()
            })
            .run(|mut buf| {
                brag::composite::premultiply_f32(&mut buf).unwrap();
                black_box(buf)
            })
        });

        g.bench("naive-f32-scalar", |b| {
            b.with_input(|| {
                (0..PIXELS * 4)
                    .map(|i| (i % 251) as f32 / 255.0)
                    .collect::<Vec<f32>>()
            })
            .run(|mut buf| {
                for px in buf.chunks_exact_mut(4) {
                    let a = px[2];
                    px[0] *= a;
                    px[1] *= a;
                    px[3] *= a;
                }
                black_box(buf)
            })
        });
    });
}

zenbench::main!(bench_src_over_u8, bench_src_over_f32, bench_premultiply);
