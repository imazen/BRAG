//! Lossless transcode: convert any JPEG to sequential with restart markers.
//!
//! Usage: cargo run --example transcode_rst --release --features composite,swizzle -- input.jpg output.jpg

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.jpg> <output.jpg>", args[0]);
        std::process::exit(1);
    }

    let input = std::fs::read(&args[1]).expect("failed to read input");
    eprintln!("Input: {} ({} bytes)", args[1], input.len());

    // Probe input
    let info = zenjpeg::decoder::Decoder::new()
        .read_info(&input)
        .expect("failed to read JPEG info");
    eprintln!(
        "  {}x{} {:?} {:?}",
        info.dimensions.width, info.dimensions.height, info.color_space, info.mode
    );

    // Lossless restructure: sequential + RST every 4 MCU rows
    let config = zenjpeg::lossless::RestructureConfig {
        output_mode: zenjpeg::lossless::OutputMode::Sequential,
        restart_interval: zenjpeg::lossless::RestartInterval::EveryMcuRows(4),
        transform: None,
    };

    let output = zenjpeg::lossless::restructure(&input, &config, enough::Unstoppable)
        .expect("transcode failed");

    eprintln!("Output: {} ({} bytes)", args[2], output.len());
    std::fs::write(&args[2], &output).expect("failed to write output");
    eprintln!("Done.");
}
