//! JPEG encoder quality vs size comparison using butteraugli.
//!
//! Encodes the same 4K source at multiple quality levels with
//! zenjpeg, mozjpeg, and jpeg-encoder, then measures perceptual
//! quality via butteraugli score.
//!
//! Run: cargo run --example jpeg_quality --release --features composite,swizzle
//!
//! Output: TSV table of (encoder, quality, size_bytes, butteraugli_score)

use enough::Unstoppable;
use imgref::ImgVec;
use rgb::RGB8;
use std::io::Cursor;

const W: u32 = 1920;
const H: u32 = 1080;

fn make_source_rgb(w: u32, h: u32) -> Vec<u8> {
    let n = (w * h) as usize;
    let mut buf = Vec::with_capacity(n * 3);
    // Gradient + noise pattern for realistic compression behavior
    for i in 0..n {
        let x = (i % w as usize) as f32 / w as f32;
        let y = (i / w as usize) as f32 / h as f32;
        let noise = ((i * 7 + 13) % 31) as f32 / 31.0 * 0.1;
        let r = ((x * 0.8 + noise) * 255.0).clamp(0.0, 255.0) as u8;
        let g = ((y * 0.7 + noise) * 255.0).clamp(0.0, 255.0) as u8;
        let b = (((x + y) * 0.5 + noise) * 255.0).clamp(0.0, 255.0) as u8;
        buf.extend_from_slice(&[r, g, b]);
    }
    buf
}

fn decode_jpeg_to_rgb(jpeg: &[u8]) -> Vec<u8> {
    let result = zenjpeg::decoder::Decoder::new()
        .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
        .decode(jpeg, Unstoppable)
        .unwrap();
    result.into_pixels_u8().unwrap()
}

fn rgb_to_imgref(rgb: &[u8], w: usize, h: usize) -> ImgVec<RGB8> {
    let pixels: Vec<RGB8> = rgb
        .chunks_exact(3)
        .map(|c| RGB8 {
            r: c[0],
            g: c[1],
            b: c[2],
        })
        .collect();
    ImgVec::new(pixels, w, h)
}

fn encode_zenjpeg(rgb: &[u8], w: u32, h: u32, quality: u8) -> Vec<u8> {
    let config = zenjpeg::encoder::EncoderConfig::ycbcr(
        quality,
        zenjpeg::encoder::ChromaSubsampling::Quarter,
    );
    let mut enc = config
        .encode_from_bytes(w, h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .unwrap();
    enc.push_packed(rgb, Unstoppable).unwrap();
    enc.finish().unwrap()
}

fn encode_mozjpeg(rgb: &[u8], w: u32, h: u32, quality: u8) -> Vec<u8> {
    let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    comp.set_size(w as usize, h as usize);
    comp.set_quality(quality as f32);
    let mut started = comp.start_compress(Vec::new()).unwrap();
    started.write_scanlines(rgb).unwrap();
    started.finish().unwrap()
}

fn encode_jpeg_encoder(rgb: &[u8], w: u32, h: u32, quality: u8) -> Vec<u8> {
    let mut buf = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut buf, quality);
    encoder
        .encode(rgb, w as u16, h as u16, jpeg_encoder::ColorType::Rgb)
        .unwrap();
    buf
}

fn main() {
    let source = make_source_rgb(W, H);
    let source_img = rgb_to_imgref(&source, W as usize, H as usize);
    let params = butteraugli::ButteraugliParams::default();

    println!("encoder\tquality\tsize_bytes\tsize_kb\tbutteraugli");

    let qualities = [60, 70, 75, 80, 85, 90, 95];

    for &q in &qualities {
        // zenjpeg
        let jpeg = encode_zenjpeg(&source, W, H, q);
        let decoded = decode_jpeg_to_rgb(&jpeg);
        let decoded_img = rgb_to_imgref(&decoded, W as usize, H as usize);
        let score = butteraugli::butteraugli(source_img.as_ref(), decoded_img.as_ref(), &params)
            .unwrap()
            .score;
        println!(
            "zenjpeg\t{q}\t{}\t{:.1}\t{score:.4}",
            jpeg.len(),
            jpeg.len() as f64 / 1024.0
        );

        // mozjpeg
        let jpeg = encode_mozjpeg(&source, W, H, q);
        let decoded = decode_jpeg_to_rgb(&jpeg);
        let decoded_img = rgb_to_imgref(&decoded, W as usize, H as usize);
        let score = butteraugli::butteraugli(source_img.as_ref(), decoded_img.as_ref(), &params)
            .unwrap()
            .score;
        println!(
            "mozjpeg\t{q}\t{}\t{:.1}\t{score:.4}",
            jpeg.len(),
            jpeg.len() as f64 / 1024.0
        );

        // jpeg-encoder
        let jpeg = encode_jpeg_encoder(&source, W, H, q);
        let decoded = decode_jpeg_to_rgb(&jpeg);
        let decoded_img = rgb_to_imgref(&decoded, W as usize, H as usize);
        let score = butteraugli::butteraugli(source_img.as_ref(), decoded_img.as_ref(), &params)
            .unwrap()
            .score;
        println!(
            "jpeg-encoder\t{q}\t{}\t{:.1}\t{score:.4}",
            jpeg.len(),
            jpeg.len() as f64 / 1024.0
        );
    }
}
