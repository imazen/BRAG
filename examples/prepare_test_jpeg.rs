//! Decode → resize (crop-constrain to 3840×2160) → encode 4:4:4 q85 with RSTs.
//!
//! Usage: cargo run --example prepare_test_jpeg --release --features composite,swizzle -- input.jpg output.jpg

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.jpg> <output.jpg>", args[0]);
        std::process::exit(1);
    }

    let input_data = std::fs::read(&args[1]).expect("failed to read input");
    eprintln!("Input: {} ({} bytes)", args[1], input_data.len());

    // Decode to RGBA u8
    let result = zenjpeg::decoder::Decoder::new()
        .output_target(zenjpeg::decoder::OutputTarget::Srgb8)
        .decode(&input_data, enough::Unstoppable)
        .expect("decode failed");

    let in_w = result.width;
    let in_h = result.height;
    let rgb = result.into_pixels_u8().expect("expected u8 output");
    eprintln!("  Decoded: {in_w}x{in_h} ({} bytes RGB)", rgb.len());

    // Expand RGB to RGBA (zenresize needs RGBA)
    let mut rgba = Vec::with_capacity(rgb.len() / 3 * 4);
    for c in rgb.chunks_exact(3) {
        rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }

    // Resize to 3840x2160 using Lanczos (crop-constrain: resize to cover, then crop)
    let out_w: u32 = 3840;
    let out_h: u32 = 2160;

    // Compute scale to cover the target dimensions
    let scale_w = out_w as f64 / in_w as f64;
    let scale_h = out_h as f64 / in_h as f64;
    let scale = scale_w.max(scale_h);

    let resize_w = ((in_w as f64 * scale).ceil() as u32).max(out_w);
    let resize_h = ((in_h as f64 * scale).ceil() as u32).max(out_h);

    eprintln!(
        "  Resize: {in_w}x{in_h} → {resize_w}x{resize_h} (cover), then crop to {out_w}x{out_h}"
    );

    let config = zenresize::ResizeConfig::builder(in_w, in_h, resize_w, resize_h)
        .filter(zenresize::Filter::Lanczos)
        .format(zenresize::PixelDescriptor::RGBA8_SRGB)
        .build();
    let mut resizer = zenresize::Resizer::new(&config);
    let resized = resizer.resize(&rgba);

    // Crop to exact 3840x2160 from center
    let crop_x = (resize_w - out_w) / 2;
    let crop_y = (resize_h - out_h) / 2;
    let stride = (resize_w * 4) as usize;

    let mut cropped_rgb = Vec::with_capacity((out_w * out_h * 3) as usize);
    for y in 0..out_h as usize {
        let row_start = (crop_y as usize + y) * stride + (crop_x as usize) * 4;
        for x in 0..out_w as usize {
            let px = row_start + x * 4;
            cropped_rgb.extend_from_slice(&resized[px..px + 3]); // RGB only, drop A
        }
    }

    eprintln!(
        "  Cropped: {out_w}x{out_h} ({} bytes RGB)",
        cropped_rgb.len()
    );

    // Encode as JPEG: 4:4:4, q85, with restart markers
    let enc_config =
        zenjpeg::encoder::EncoderConfig::ycbcr(85, zenjpeg::encoder::ChromaSubsampling::None);
    let mut enc = enc_config
        .encode_from_bytes(out_w, out_h, zenjpeg::encoder::PixelLayout::Rgb8Srgb)
        .expect("encoder init failed");
    enc.push_packed(&cropped_rgb, enough::Unstoppable)
        .expect("encode failed");
    let jpeg_out = enc.finish().expect("finish failed");

    eprintln!("Output: {} ({} bytes)", args[2], jpeg_out.len());
    std::fs::write(&args[2], &jpeg_out).expect("failed to write output");
    eprintln!("Done.");
}
