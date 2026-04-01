//! Pipeline benchmark: decode → swizzle → composite.
//!
//! Compares zen crates + brag vs zune-* vs image crate
//! on the same PNG (512x512 with alpha) and JPEG (3840x2160) source data.
//!
//! Run: cargo bench --bench pipeline --features composite,swizzle

use enough::Unstoppable;
use imgref::ImgVec;
use rgb::Rgba;
use std::io::Cursor;
use std::sync::Arc;
use zenbench::prelude::*;

// ── Image dimensions ───────────────────────────────────────────────

const PNG_W: u32 = 512;
const PNG_H: u32 = 512;
const JPEG_W: u32 = 3840;
const JPEG_H: u32 = 2160;

// ── Test data generation (run once at startup) ─────────────────────

fn make_fg_rgba(w: u32, h: u32) -> Vec<u8> {
    let n = (w * h) as usize;
    let mut buf = Vec::with_capacity(n * 4);
    for i in 0..n {
        let x = (i % w as usize) as u8;
        let y = (i / w as usize) as u8;
        let a = (x.wrapping_add(y)) | 0x40;
        buf.extend_from_slice(&[x, y, x ^ y, a]);
    }
    buf
}

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
    config.compression = zenpng::Compression::Fast; // Level 1 — realistic
    zenpng::encode_rgba8(img.as_ref(), None, &config, &Unstoppable, &Unstoppable).unwrap()
}

fn encode_test_jpeg(w: u32, h: u32) -> Vec<u8> {
    let rgb = make_bg_rgb(w, h);
    let config =
        zenjpeg::encoder::EncoderConfig::ycbcr(85, zenjpeg::encoder::ChromaSubsampling::Quarter);
    let mut enc = config
        .encode_from_bytes(w, h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .unwrap();
    enc.push_packed(&rgb, Unstoppable).unwrap();
    enc.finish().unwrap()
}

// ── Decode helpers ─────────────────────────────────────────────────

/// zen pipeline: decode JPEG → RGB → expand to RGBA → BRAG
fn zen_decode_jpeg_to_brag(jpeg: &[u8]) -> Vec<u8> {
    let result = zenjpeg::decoder::Decoder::new()
        .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
        .decode(jpeg, Unstoppable)
        .unwrap();
    let rgb = result.pixels_u8().unwrap();
    let mut pixels = Vec::with_capacity(rgb.len() / 3 * 4);
    for c in rgb.chunks_exact(3) {
        pixels.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }
    brag::swizzle::rgba_to_brag_inplace(&mut pixels).unwrap();
    pixels
}

/// zen pipeline: decode PNG → RGBA → BRAG → premultiply
fn zen_decode_png_to_premul_brag(png: &[u8]) -> Vec<u8> {
    let output = zenpng::decode(png, &zenpng::PngDecodeConfig::default(), &Unstoppable).unwrap();
    let mut pixels = output.pixels.into_vec();
    brag::swizzle::rgba_to_brag_inplace(&mut pixels).unwrap();
    brag::composite::premultiply(&mut pixels).unwrap();
    pixels
}

// ── Benchmarks ─────────────────────────────────────────────────────

fn bench_jpeg_decode(suite: &mut Suite) {
    let jpeg = Arc::new(encode_test_jpeg(JPEG_W, JPEG_H));
    let bytes = (JPEG_W as u64) * (JPEG_H as u64) * 3; // RGB output

    let j1 = jpeg.clone();
    let j2 = jpeg.clone();
    let j3 = jpeg.clone();

    suite.group("jpeg_decode_4k", move |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("zenjpeg");

        g.bench("zenjpeg", move |b| {
            let data = j1.clone();
            b.iter(move || {
                let r = zenjpeg::decoder::Decoder::new()
                    .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
                    .decode(&data, Unstoppable)
                    .unwrap();
                black_box(r.into_pixels_u8().unwrap())
            })
        });

        g.bench("zune-jpeg", move |b| {
            let data = j2.clone();
            b.iter(move || {
                let mut dec = zune_jpeg::JpegDecoder::new(Cursor::new(&*data));
                let pixels = dec.decode().unwrap();
                black_box(pixels)
            })
        });

        g.bench("image", move |b| {
            let data = j3.clone();
            b.iter(move || {
                let img = image::ImageReader::new(Cursor::new(&*data))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();
                black_box(img.to_rgb8().into_raw())
            })
        });
    });
}

fn bench_png_decode(suite: &mut Suite) {
    let png = Arc::new(encode_test_png(PNG_W, PNG_H));
    let bytes = (PNG_W as u64) * (PNG_H as u64) * 4; // RGBA output

    let p1 = png.clone();
    let p2 = png.clone();
    let p3 = png.clone();

    suite.group("png_decode_512x512", move |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("zenpng");

        g.bench("zenpng", move |b| {
            let data = p1.clone();
            b.iter(move || {
                let out = zenpng::decode(&data, &zenpng::PngDecodeConfig::default(), &Unstoppable)
                    .unwrap();
                black_box(out.pixels.into_vec())
            })
        });

        g.bench("zune-png", move |b| {
            let data = p2.clone();
            b.iter(move || {
                let mut dec = zune_png::PngDecoder::new(Cursor::new(&*data));
                let pixels = dec.decode_raw().unwrap();
                black_box(pixels)
            })
        });

        g.bench("image", move |b| {
            let data = p3.clone();
            b.iter(move || {
                let img = image::ImageReader::new(Cursor::new(&*data))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();
                black_box(img.to_rgba8().into_raw())
            })
        });
    });
}

fn bench_full_pipeline(suite: &mut Suite) {
    // Full pipeline: decode 4K JPEG bg + 512x512 PNG fg → composite
    // We composite the PNG over the top-left 512x512 of the JPEG.
    let jpeg = Arc::new(encode_test_jpeg(JPEG_W, JPEG_H));
    let png = Arc::new(encode_test_png(PNG_W, PNG_H));

    let composite_pixels = (PNG_W as u64) * (PNG_H as u64);
    let composite_bytes = composite_pixels * 4;

    // Clone for later groups before moving into first closure
    let j3 = jpeg.clone();
    let p3 = png.clone();

    // ── zen + brag pipeline ────────────────────────────────────
    let j1 = jpeg.clone();
    let p1 = png.clone();
    suite.group("full_pipeline_decode_composite", move |g| {
        g.throughput(Throughput::Bytes(composite_bytes));
        g.baseline("zen+brag");

        g.bench("zen+brag", move |b| {
            let jpeg = j1.clone();
            let png = p1.clone();
            b.iter(move || {
                // Decode JPEG 4K → RGBA → BRAG (only first 512x512 used)
                let bg_full = zen_decode_jpeg_to_brag(&jpeg);
                // Take first 512 rows (stride = JPEG_W * 4, width = PNG_W * 4)
                let mut bg_crop: Vec<u8> = Vec::with_capacity((PNG_W * PNG_H * 4) as usize);
                let stride = (JPEG_W * 4) as usize;
                for y in 0..PNG_H as usize {
                    let start = y * stride;
                    bg_crop.extend_from_slice(&bg_full[start..start + (PNG_W * 4) as usize]);
                }

                // Decode PNG → BRAG → premultiply
                let fg = zen_decode_png_to_premul_brag(&png);

                // Composite
                brag::composite::src_over(&fg, &mut bg_crop).unwrap();
                black_box(bg_crop)
            })
        });

        // ── zune codecs + sw-composite pipeline ──────────────────
        let j2 = jpeg.clone();
        let p2 = png.clone();
        g.bench("zune+sw-composite", move |b| {
            let jpeg = j2.clone();
            let png = p2.clone();
            b.iter(move || {
                // Decode JPEG with zune-jpeg → RGB
                let mut jdec = zune_jpeg::JpegDecoder::new(Cursor::new(&*jpeg));
                let jpeg_rgb = jdec.decode().unwrap();
                let jpeg_w = JPEG_W as usize;

                // Expand RGB→ARGB packed u32 (sw-composite format: 0xAARRGGBB)
                let mut bg_argb: Vec<u32> = jpeg_rgb
                    .chunks_exact(3)
                    .map(|c| {
                        0xFF00_0000 | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | c[2] as u32
                    })
                    .collect();

                // Decode PNG with zune-png → RGBA
                let mut pdec = zune_png::PngDecoder::new(Cursor::new(&*png));
                let png_rgba = pdec.decode_raw().unwrap();

                // Convert PNG RGBA → premul ARGB packed u32
                let fg_argb: Vec<u32> = png_rgba
                    .chunks_exact(4)
                    .map(|c| {
                        let a = c[3] as u32;
                        let r = ((c[0] as u32 * a + 128) / 255) as u32;
                        let g = ((c[1] as u32 * a + 128) / 255) as u32;
                        let b = ((c[2] as u32 * a + 128) / 255) as u32;
                        (a << 24) | (r << 16) | (g << 8) | b
                    })
                    .collect();

                // Composite 512x512 over top-left of 4K using sw-composite
                for y in 0..PNG_H as usize {
                    let bg_start = y * jpeg_w;
                    for x in 0..PNG_W as usize {
                        let fg_px = fg_argb[y * PNG_W as usize + x];
                        bg_argb[bg_start + x] = sw_composite::over(fg_px, bg_argb[bg_start + x]);
                    }
                }
                black_box(bg_argb)
            })
        });

        // ── image crate pipeline ───────────────────────────────
        let j3i = jpeg.clone();
        let p3i = png.clone();
        g.bench("image", move |b| {
            let jpeg = j3i.clone();
            let png = p3i.clone();
            b.iter(move || {
                let bg = image::ImageReader::new(Cursor::new(&*jpeg))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap()
                    .to_rgba8();
                let fg = image::ImageReader::new(Cursor::new(&*png))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap()
                    .to_rgba8();

                let mut bg = image::DynamicImage::ImageRgba8(bg);
                image::imageops::overlay(&mut bg, &fg, 0, 0);
                black_box(bg.to_rgba8().into_raw())
            })
        });
    });

    // ── Composite-only (pre-decoded, 4K BRAG buffers) ──────────
    let bg_4k = Arc::new(zen_decode_jpeg_to_brag(&j3));
    let fg_512 = Arc::new(zen_decode_png_to_premul_brag(&p3));

    suite.group("composite_only_512x512", move |g| {
        g.throughput(Throughput::Bytes(composite_bytes));

        g.bench("brag_src_over", move |b| {
            let fg = fg_512.clone();
            let bg_full = bg_4k.clone();
            b.with_input(move || {
                // Crop 512x512 from top-left of 4K
                let stride = (JPEG_W * 4) as usize;
                let mut crop = Vec::with_capacity((PNG_W * PNG_H * 4) as usize);
                for y in 0..PNG_H as usize {
                    let start = y * stride;
                    crop.extend_from_slice(&bg_full[start..start + (PNG_W * 4) as usize]);
                }
                ((*fg).clone(), crop)
            })
            .run(|(src, mut dst)| {
                brag::composite::src_over(&src, &mut dst).unwrap();
                black_box(dst)
            })
        });
    });
}

zenbench::main!(bench_jpeg_decode, bench_png_decode, bench_full_pipeline);
