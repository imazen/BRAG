# BRAG — justfile
# Run `just` to see all available commands.

# Run all benchmarks and save markdown reports
bench: bench-compositing bench-pipeline bench-quality
    @echo "All benchmarks complete. Results in BENCHMARKS.md"

# Compositing benchmark: brag vs sw-composite vs tiny-skia vs alpha-blend
bench-compositing:
    cargo bench --bench compositing 

# Compositing benchmark (markdown output)
bench-compositing-md:
    cargo bench --bench compositing  -- --format=md

# Pipeline benchmark: zen+brag vs zune+sw-composite vs image
bench-pipeline:
    cargo bench --bench pipeline --features swizzle

# Pipeline benchmark (markdown output)
bench-pipeline-md:
    cargo bench --bench pipeline --features swizzle -- --format=md

# Butteraugli quality-vs-size analysis for JPEG encoders
bench-quality:
    cargo run --example jpeg_quality --release --features swizzle

# Run all tests
test:
    cargo test --features swizzle

# Run clippy
clippy:
    cargo clippy --features swizzle

# Format code
fmt:
    cargo fmt

# Normalize any JPEG to sequential 4:4:4 q85 with RST markers (for benchmarking)
# Decode → Lanczos resize (crop-constrain) → encode sequential 4:4:4 q85
# Usage: just normalize-jpeg input.jpg output.jpg [3840x2160]
normalize-jpeg input output dims="3840x2160":
    cargo run --example prepare_test_jpeg --release --features swizzle -- {{input}} {{output}} {{dims}}

# Full CI check (fmt + clippy + test)
ci: fmt clippy test
