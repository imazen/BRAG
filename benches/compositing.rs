//! Compositing benchmark: brag vs sw-composite vs alpha-blend.
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

fn make_premul_f32(n: usize) -> Vec<alpha_blend::rgba::F32x4Rgba> {
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

// ── Benchmarks ─────────────────────────────────────────────────────

fn bench_src_over(suite: &mut Suite) {
    const SIZES: &[(usize, &str)] = &[(256 * 256, "256x256"), (1024 * 1024, "1024x1024")];

    for &(pixels, label) in SIZES {
        let bytes = (pixels * 4) as u64;

        suite.group(format!("src_over_{label}"), move |g| {
            g.throughput(Throughput::Bytes(bytes));
            g.baseline("brag");

            g.bench("brag", move |b| {
                b.with_input(move || (make_premul_brag(pixels), make_premul_brag(pixels)))
                    .run(|(src, mut dst)| {
                        brag::composite::src_over(&src, &mut dst).unwrap();
                        black_box(dst)
                    })
            });

            g.bench("sw-composite", move |b| {
                b.with_input(move || (make_premul_argb_u32(pixels), make_premul_argb_u32(pixels)))
                    .run(|(src, mut dst)| {
                        for (s, d) in src.iter().zip(dst.iter_mut()) {
                            *d = sw_composite::over(*s, *d);
                        }
                        black_box(dst)
                    })
            });

            g.bench("sw-composite-exact", move |b| {
                b.with_input(move || (make_premul_argb_u32(pixels), make_premul_argb_u32(pixels)))
                    .run(|(src, mut dst)| {
                        for (s, d) in src.iter().zip(dst.iter_mut()) {
                            *d = sw_composite::over_exact(*s, *d);
                        }
                        black_box(dst)
                    })
            });

            g.bench("alpha-blend-f32", move |b| {
                b.with_input(move || (make_premul_f32(pixels), make_premul_f32(pixels)))
                    .run(|(src, mut dst)| {
                        let mode = alpha_blend::BlendMode::SourceOver;
                        for (s, d) in src.iter().zip(dst.iter_mut()) {
                            *d = mode.apply(*s, *d);
                        }
                        black_box(dst)
                    })
            });

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

fn bench_premultiply(suite: &mut Suite) {
    const PIXELS: usize = 1024 * 1024;
    let bytes = (PIXELS * 4) as u64;

    suite.group("premultiply_1024x1024", |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("brag");

        g.bench("brag", |b| {
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
}

zenbench::main!(bench_src_over, bench_premultiply);
