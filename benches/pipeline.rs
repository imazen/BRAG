//! Pipeline benchmark: decode → composite, plus codec comparisons.
//!
//! Compares zen+brag vs zune+sw-composite vs image crate pipelines,
//! plus JPEG encode/decode speed across zenjpeg, mozjpeg, zune-jpeg,
//! jpeg-encoder, and the image crate.
//!
//! Run: cargo bench --bench pipeline --features composite,swizzle

use enough::Unstoppable;
use imgref::ImgVec;
use rgb::Rgba;
use std::io::Cursor;
use std::sync::Arc;
use zenbench::prelude::*;

const PNG_W: u32 = 512;
const PNG_H: u32 = 512;
const JPEG_W: u32 = 3840;
const JPEG_H: u32 = 2160;

// ── Test data generation ───────────────────────────────────────────

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
    config.compression = zenpng::Compression::Fast;
    zenpng::encode_rgba8(img.as_ref(), None, &config, &Unstoppable, &Unstoppable).unwrap()
}

// ── JPEG encoders (all at quality 85, same source RGB data) ────────

fn encode_jpeg_zenjpeg(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let config =
        zenjpeg::encoder::EncoderConfig::ycbcr(85, zenjpeg::encoder::ChromaSubsampling::Quarter);
    let mut enc = config
        .encode_from_bytes(w, h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .unwrap();
    enc.push_packed(rgb, Unstoppable).unwrap();
    enc.finish().unwrap()
}

fn encode_jpeg_zenjpeg_parallel(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let config =
        zenjpeg::encoder::EncoderConfig::ycbcr(85, zenjpeg::encoder::ChromaSubsampling::Quarter)
            .parallel(zenjpeg::encoder::ParallelEncoding::Auto);
    let mut enc = config
        .encode_from_bytes(w, h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .unwrap();
    enc.push_packed(rgb, Unstoppable).unwrap();
    enc.finish().unwrap()
}

fn encode_jpeg_zenjpeg_fixed(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let config =
        zenjpeg::encoder::EncoderConfig::ycbcr(85, zenjpeg::encoder::ChromaSubsampling::Quarter)
            .progressive(false)
            .huffman(zenjpeg::encoder::HuffmanStrategy::Fixed);
    let mut enc = config
        .encode_from_bytes(w, h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .unwrap();
    enc.push_packed(rgb, Unstoppable).unwrap();
    enc.finish().unwrap()
}

fn encode_jpeg_mozjpeg(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    comp.set_size(w as usize, h as usize);
    comp.set_quality(85.0);
    let mut started = comp.start_compress(Vec::new()).unwrap();
    started.write_scanlines(rgb).unwrap();
    started.finish().unwrap()
}

fn encode_jpeg_encoder(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut buf, 85);
    encoder
        .encode(rgb, w as u16, h as u16, jpeg_encoder::ColorType::Rgb)
        .unwrap();
    buf
}

// ── Decode helpers ─────────────────────────────────────────────────

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

fn zen_decode_png_to_premul_brag(png: &[u8]) -> Vec<u8> {
    let output = zenpng::decode(png, &zenpng::PngDecodeConfig::default(), &Unstoppable).unwrap();
    let mut pixels = output.pixels.into_vec();
    brag::swizzle::rgba_to_brag_inplace(&mut pixels).unwrap();
    brag::composite::premultiply(&mut pixels).unwrap();
    pixels
}

// ── Benchmarks ─────────────────────────────────────────────────────

fn bench_jpeg_decode(suite: &mut Suite) {
    // Encode with zenjpeg (all decoders decode the same file)
    let rgb = make_bg_rgb(JPEG_W, JPEG_H);
    let jpeg = Arc::new(encode_jpeg_zenjpeg(&rgb, JPEG_W, JPEG_H));
    let bytes = (JPEG_W as u64) * (JPEG_H as u64) * 3;

    std::eprintln!(
        "JPEG 4K encoded size: {} bytes ({:.1} KB)",
        jpeg.len(),
        jpeg.len() as f64 / 1024.0
    );

    let j1 = jpeg.clone();
    let j2 = jpeg.clone();
    let j3 = jpeg.clone();
    let j4 = jpeg.clone();

    suite.group("jpeg_decode_4k", move |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("zenjpeg");

        g.bench("zenjpeg", move |b| {
            let d = j1.clone();
            b.iter(move || {
                let r = zenjpeg::decoder::Decoder::new()
                    .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
                    .decode(&d, Unstoppable)
                    .unwrap();
                black_box(r.into_pixels_u8().unwrap())
            })
        });

        g.bench("mozjpeg", move |b| {
            let d = j2.clone();
            b.iter(move || {
                let dec = mozjpeg::Decompress::new_mem(&d).unwrap();
                let mut dec = dec.rgb().unwrap();
                let pixels: Vec<u8> = dec.read_scanlines().unwrap();
                black_box(pixels)
            })
        });

        g.bench("zune-jpeg", move |b| {
            let d = j3.clone();
            b.iter(move || {
                let mut dec = zune_jpeg::JpegDecoder::new(Cursor::new(&*d));
                black_box(dec.decode().unwrap())
            })
        });

        g.bench("image", move |b| {
            let d = j4.clone();
            b.iter(move || {
                let img = image::ImageReader::new(Cursor::new(&*d))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();
                black_box(img.to_rgb8().into_raw())
            })
        });
    });
}

fn bench_jpeg_encode(suite: &mut Suite) {
    let rgb = Arc::new(make_bg_rgb(JPEG_W, JPEG_H));
    let bytes = (JPEG_W as u64) * (JPEG_H as u64) * 3;

    let r1 = rgb.clone();
    let r2 = rgb.clone();
    let r3 = rgb.clone();
    let r4 = rgb.clone();
    let r5 = rgb.clone();

    suite.group("jpeg_encode_4k_q85", move |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("zenjpeg");

        g.bench("zenjpeg", move |b| {
            let rgb = r1.clone();
            b.iter(move || black_box(encode_jpeg_zenjpeg(&rgb, JPEG_W, JPEG_H)))
        });

        g.bench("zenjpeg-parallel", move |b| {
            let rgb = r4.clone();
            b.iter(move || black_box(encode_jpeg_zenjpeg_parallel(&rgb, JPEG_W, JPEG_H)))
        });

        g.bench("zenjpeg-fixed-huff", move |b| {
            let rgb = r5.clone();
            b.iter(move || black_box(encode_jpeg_zenjpeg_fixed(&rgb, JPEG_W, JPEG_H)))
        });

        g.bench("mozjpeg", move |b| {
            let rgb = r2.clone();
            b.iter(move || black_box(encode_jpeg_mozjpeg(&rgb, JPEG_W, JPEG_H)))
        });

        g.bench("jpeg-encoder", move |b| {
            let rgb = r3.clone();
            b.iter(move || black_box(encode_jpeg_encoder(&rgb, JPEG_W, JPEG_H)))
        });
    });

    // Report encoded sizes
    let rgb_ref = &*rgb;
    let sz_zen = encode_jpeg_zenjpeg(rgb_ref, JPEG_W, JPEG_H).len();
    let sz_par = encode_jpeg_zenjpeg_parallel(rgb_ref, JPEG_W, JPEG_H).len();
    let sz_fix = encode_jpeg_zenjpeg_fixed(rgb_ref, JPEG_W, JPEG_H).len();
    let sz_moz = encode_jpeg_mozjpeg(rgb_ref, JPEG_W, JPEG_H).len();
    let sz_enc = encode_jpeg_encoder(rgb_ref, JPEG_W, JPEG_H).len();
    std::eprintln!("\nJPEG 4K encode sizes (quality 85, 4:2:0):");
    std::eprintln!(
        "  zenjpeg:            {sz_zen:>8} bytes ({:.1} KB)",
        sz_zen as f64 / 1024.0
    );
    std::eprintln!(
        "  zenjpeg-parallel:   {sz_par:>8} bytes ({:.1} KB)",
        sz_par as f64 / 1024.0
    );
    std::eprintln!(
        "  zenjpeg-fixed-huff: {sz_fix:>8} bytes ({:.1} KB)",
        sz_fix as f64 / 1024.0
    );
    std::eprintln!(
        "  mozjpeg:            {sz_moz:>8} bytes ({:.1} KB)",
        sz_moz as f64 / 1024.0
    );
    std::eprintln!(
        "  jpeg-encoder:       {sz_enc:>8} bytes ({:.1} KB)",
        sz_enc as f64 / 1024.0
    );
}

fn bench_png_decode(suite: &mut Suite) {
    let png = Arc::new(encode_test_png(PNG_W, PNG_H));
    let bytes = (PNG_W as u64) * (PNG_H as u64) * 4;

    std::eprintln!(
        "\nPNG 512x512 encoded size: {} bytes ({:.1} KB)",
        png.len(),
        png.len() as f64 / 1024.0
    );

    let p1 = png.clone();
    let p2 = png.clone();
    let p3 = png.clone();

    suite.group("png_decode_512x512", move |g| {
        g.throughput(Throughput::Bytes(bytes));
        g.baseline("zenpng");

        g.bench("zenpng", move |b| {
            let d = p1.clone();
            b.iter(move || {
                let out =
                    zenpng::decode(&d, &zenpng::PngDecodeConfig::default(), &Unstoppable).unwrap();
                black_box(out.pixels.into_vec())
            })
        });

        g.bench("zune-png", move |b| {
            let d = p2.clone();
            b.iter(move || {
                let mut dec = zune_png::PngDecoder::new(Cursor::new(&*d));
                black_box(dec.decode_raw().unwrap())
            })
        });

        g.bench("image", move |b| {
            let d = p3.clone();
            b.iter(move || {
                let img = image::ImageReader::new(Cursor::new(&*d))
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
    let rgb = make_bg_rgb(JPEG_W, JPEG_H);
    let jpeg = Arc::new(encode_jpeg_zenjpeg(&rgb, JPEG_W, JPEG_H));
    let png = Arc::new(encode_test_png(PNG_W, PNG_H));

    let composite_bytes = (PNG_W as u64) * (PNG_H as u64) * 4;

    // Clone for later group
    let j_later = jpeg.clone();
    let p_later = png.clone();

    let j1 = jpeg.clone();
    let p1 = png.clone();
    let j2 = jpeg.clone();
    let p2 = png.clone();
    let j3 = jpeg.clone();
    let p3 = png.clone();

    suite.group("full_pipeline_4k", move |g| {
        g.throughput(Throughput::Bytes(composite_bytes));
        g.baseline("zen+brag");

        // ── zen + brag ─────────────────────────────────────────
        g.bench("zen+brag", move |b| {
            let jpeg = j1.clone();
            let png = p1.clone();
            b.iter(move || {
                let bg_full = zen_decode_jpeg_to_brag(&jpeg);
                let stride = (JPEG_W * 4) as usize;
                let mut bg_crop = Vec::with_capacity((PNG_W * PNG_H * 4) as usize);
                for y in 0..PNG_H as usize {
                    let start = y * stride;
                    bg_crop.extend_from_slice(&bg_full[start..start + (PNG_W * 4) as usize]);
                }
                let fg = zen_decode_png_to_premul_brag(&png);
                brag::composite::src_over(&fg, &mut bg_crop).unwrap();
                black_box(bg_crop)
            })
        });

        // ── zune + sw-composite ────────────────────────────────
        g.bench("zune+sw-composite", move |b| {
            let jpeg = j2.clone();
            let png = p2.clone();
            b.iter(move || {
                let mut jdec = zune_jpeg::JpegDecoder::new(Cursor::new(&*jpeg));
                let jpeg_rgb = jdec.decode().unwrap();
                let jpeg_w = JPEG_W as usize;

                let mut bg_argb: Vec<u32> = jpeg_rgb
                    .chunks_exact(3)
                    .map(|c| {
                        0xFF00_0000 | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | c[2] as u32
                    })
                    .collect();

                let mut pdec = zune_png::PngDecoder::new(Cursor::new(&*png));
                let png_rgba = pdec.decode_raw().unwrap();

                let fg_argb: Vec<u32> = png_rgba
                    .chunks_exact(4)
                    .map(|c| {
                        let a = c[3] as u32;
                        let r = (c[0] as u32 * a + 128) / 255;
                        let g = (c[1] as u32 * a + 128) / 255;
                        let b = (c[2] as u32 * a + 128) / 255;
                        (a << 24) | (r << 16) | (g << 8) | b
                    })
                    .collect();

                for y in 0..PNG_H as usize {
                    let bg_start = y * jpeg_w;
                    for x in 0..PNG_W as usize {
                        bg_argb[bg_start + x] = sw_composite::over(
                            fg_argb[y * PNG_W as usize + x],
                            bg_argb[bg_start + x],
                        );
                    }
                }
                black_box(bg_argb)
            })
        });

        // ── image crate ────────────────────────────────────────
        g.bench("image", move |b| {
            let jpeg = j3.clone();
            let png = p3.clone();
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

    // ── Composite-only ─────────────────────────────────────────
    let bg_4k = Arc::new(zen_decode_jpeg_to_brag(&j_later));
    let fg_512 = Arc::new(zen_decode_png_to_premul_brag(&p_later));

    suite.group("composite_only_512x512", move |g| {
        g.throughput(Throughput::Bytes(composite_bytes));

        let fg = fg_512.clone();
        let bg = bg_4k.clone();
        g.bench("brag_src_over", move |b| {
            let fg = fg.clone();
            let bg = bg.clone();
            b.with_input(move || {
                let stride = (JPEG_W * 4) as usize;
                let mut crop = Vec::with_capacity((PNG_W * PNG_H * 4) as usize);
                for y in 0..PNG_H as usize {
                    let start = y * stride;
                    crop.extend_from_slice(&bg[start..start + (PNG_W * 4) as usize]);
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

zenbench::main!(
    bench_jpeg_decode,
    bench_jpeg_encode,
    bench_png_decode,
    bench_full_pipeline
);
