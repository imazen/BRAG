//! Pipeline benchmark: decode → (resize?) → swizzle → composite.
//!
//! Benchmarks the full image pipeline using zen crates:
//! - zenpng for PNG encode/decode (level 0 = uncompressed)
//! - zenjpeg for JPEG encode/decode (parallel vs sequential)
//! - zenresize for image resizing
//! - brag swizzle + composite
//!
//! Run: cargo bench --bench pipeline --features composite,swizzle

use enough::Unstoppable;
use imgref::ImgVec;
use rgb::Rgba;
use zenbench::prelude::*;

const W: u32 = 512;
const H: u32 = 512;
const PIXELS: usize = (W * H) as usize;

// ── Test data generation (run once, outside timing) ────────────────

/// Generate RGBA pixels with varying alpha (foreground layer).
fn make_fg_rgba(w: u32, h: u32) -> Vec<u8> {
    let n = (w * h) as usize;
    let mut buf = Vec::with_capacity(n * 4);
    for i in 0..n {
        let x = (i % w as usize) as u8;
        let y = (i / w as usize) as u8;
        let a = (x.wrapping_add(y)) | 0x40; // 64–255, semi-transparent
        let r = x;
        let g = y;
        let b = x ^ y;
        buf.extend_from_slice(&[r, g, b, a]);
    }
    buf
}

/// Generate RGB pixels (background layer — no alpha, opaque JPEG).
fn make_bg_rgb(w: u32, h: u32) -> Vec<u8> {
    let n = (w * h) as usize;
    let mut buf = Vec::with_capacity(n * 3);
    for i in 0..n {
        let x = (i % w as usize) as u8;
        let y = (i / w as usize) as u8;
        buf.extend_from_slice(&[x, y, x.wrapping_mul(y)]);
    }
    buf
}

/// Encode test foreground as PNG (uncompressed for speed).
fn encode_test_png(w: u32, h: u32) -> Vec<u8> {
    let rgba = make_fg_rgba(w, h);
    let pixels: Vec<Rgba<u8>> = rgba
        .chunks_exact(4)
        .map(|c| Rgba {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        })
        .collect();
    let img = ImgVec::new(pixels, w as usize, h as usize);
    let mut config = zenpng::EncodeConfig::default();
    config.compression = zenpng::Compression::None;
    zenpng::encode_rgba8(img.as_ref(), None, &config, &Unstoppable, &Unstoppable).unwrap()
}

/// Encode test background as JPEG (quality 90).
fn encode_test_jpeg(w: u32, h: u32) -> Vec<u8> {
    let rgb = make_bg_rgb(w, h);
    let config =
        zenjpeg::encoder::EncoderConfig::ycbcr(90, zenjpeg::encoder::ChromaSubsampling::None);
    let mut enc = config
        .encode_from_bytes(w, h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .unwrap();
    enc.push_packed(&rgb, Unstoppable).unwrap();
    enc.finish().unwrap()
}

/// Decode JPEG → RGBA u8, then convert to BRAG and premultiply (opaque).
fn jpeg_to_premul_brag(jpeg: &[u8]) -> Vec<u8> {
    let result = zenjpeg::decoder::Decoder::new()
        .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
        .decode(jpeg, Unstoppable)
        .unwrap();
    // Decoder outputs RGB; expand to RGBA with opaque alpha
    let rgb = result.pixels_u8().unwrap();
    let mut pixels = Vec::with_capacity(rgb.len() / 3 * 4);
    for c in rgb.chunks_exact(3) {
        pixels.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }
    brag::swizzle::rgba_to_brag_inplace(&mut pixels).unwrap();
    pixels
}

/// Decode PNG → RGBA u8, then convert to BRAG and premultiply.
fn png_to_premul_brag(png: &[u8]) -> Vec<u8> {
    let output = zenpng::decode(png, &zenpng::PngDecodeConfig::default(), &Unstoppable).unwrap();
    let mut pixels = output.pixels.into_vec();
    brag::swizzle::rgba_to_brag_inplace(&mut pixels).unwrap();
    brag::composite::premultiply(&mut pixels).unwrap();
    pixels
}

// ── Benchmarks ─────────────────────────────────────────────────────

fn bench_pipeline(suite: &mut Suite) {
    // Pre-encode test images (outside all timing)
    let png_data = encode_test_png(W, H);
    let jpeg_data = encode_test_jpeg(W, H);
    let png_data2 = png_data.clone();
    let jpeg_data2 = jpeg_data.clone();
    let png_data3 = png_data.clone();
    let jpeg_data3 = jpeg_data.clone();
    let jpeg_data4 = jpeg_data.clone();

    // Pre-decode for composite-only benchmark
    let fg_brag = std::sync::Arc::new(png_to_premul_brag(&png_data));
    let bg_brag = std::sync::Arc::new(jpeg_to_premul_brag(&jpeg_data));

    let bytes = (PIXELS * 4) as u64;

    // ── Group 1: Composite only (pre-decoded buffers) ──────────
    let fg1 = fg_brag.clone();
    let bg1 = bg_brag.clone();
    suite.group("pipeline_composite_only", move |g| {
        g.throughput(Throughput::Bytes(bytes));

        g.bench("src_over_512x512", move |b| {
            let fg = fg1.clone();
            let bg = bg1.clone();
            b.with_input(move || ((*fg).clone(), (*bg).clone()))
                .run(|(src, mut dst)| {
                    brag::composite::src_over(&src, &mut dst).unwrap();
                    black_box(dst)
                })
        });
    });

    // ── Group 2: Decode + composite ────────────────────────────
    suite.group("pipeline_decode_composite", move |g| {
        g.throughput(Throughput::Bytes(bytes));

        g.bench("png+jpeg_decode_then_composite", move |b| {
            let png = png_data2.clone();
            let jpeg = jpeg_data2.clone();
            b.iter(move || {
                let fg = png_to_premul_brag(&png);
                let mut bg = jpeg_to_premul_brag(&jpeg);
                brag::composite::src_over(&fg, &mut bg).unwrap();
                black_box(bg)
            })
        });
    });

    // ── Group 3: Decode + resize + composite ───────────────────
    suite.group("pipeline_decode_resize_composite", move |g| {
        g.throughput(Throughput::Bytes(bytes));

        g.bench("jpeg_decode_resize_256+png_composite", move |b| {
            let png = png_data3.clone();
            let jpeg = jpeg_data3.clone();
            b.iter(move || {
                // Decode JPEG (outputs RGB, expand to RGBA)
                let result = zenjpeg::decoder::Decoder::new()
                    .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
                    .decode(&jpeg, Unstoppable)
                    .unwrap();
                let rgb = result.pixels_u8().unwrap();
                let mut jpeg_pixels = Vec::with_capacity(rgb.len() / 3 * 4);
                for c in rgb.chunks_exact(3) {
                    jpeg_pixels.extend_from_slice(&[c[0], c[1], c[2], 255]);
                }

                // Resize 512→256 (Lanczos)
                let config = zenresize::ResizeConfig::builder(W, H, W / 2, H / 2)
                    .filter(zenresize::Filter::Lanczos)
                    .format(zenresize::PixelDescriptor::RGBA8_SRGB)
                    .build();
                let mut resizer = zenresize::Resizer::new(&config);
                let mut resized = resizer.resize(&jpeg_pixels);

                // Convert to BRAG
                brag::swizzle::rgba_to_brag_inplace(&mut resized).unwrap();

                // Decode PNG (512x512), convert to BRAG, premultiply
                let fg = png_to_premul_brag(&png);

                // Composite (fg is 512x512, bg is 256x256 — use the smaller)
                let composite_pixels = (W / 2 * H / 2) as usize;
                let composite_bytes = composite_pixels * 4;
                brag::composite::src_over(&fg[..composite_bytes], &mut resized).unwrap();
                black_box(resized)
            })
        });
    });

    // ── Group 4: Sequential vs parallel JPEG decode ────────────
    suite.group("jpeg_decode_512x512", move |g| {
        g.throughput(Throughput::Bytes(bytes));

        g.bench("parallel", move |b| {
            let jpeg = jpeg_data4.clone();
            b.iter(move || {
                let result = zenjpeg::decoder::Decoder::new()
                    .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
                    .decode(&jpeg, Unstoppable)
                    .unwrap();
                black_box(result.into_pixels_u8().unwrap())
            })
        });

        g.bench("sequential", move |b| {
            let jpeg = jpeg_data.clone();
            b.iter(move || {
                let result = zenjpeg::decoder::Decoder::new()
                    .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
                    .num_threads(1)
                    .decode(&jpeg, Unstoppable)
                    .unwrap();
                black_box(result.into_pixels_u8().unwrap())
            })
        });
    });
}

zenbench::main!(bench_pipeline);
