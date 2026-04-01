# BRAG — justfile
# Run `just` to see all available commands.

# Run all benchmarks and save markdown reports
bench: bench-compositing bench-pipeline bench-quality
    @echo "All benchmarks complete. Results in BENCHMARKS.md"

# Compositing benchmark: brag vs sw-composite vs tiny-skia vs alpha-blend
bench-compositing:
    cargo bench --bench compositing --features composite

# Compositing benchmark (markdown output)
bench-compositing-md:
    cargo bench --bench compositing --features composite -- --format=md

# Pipeline benchmark: zen+brag vs zune+sw-composite vs image
bench-pipeline:
    cargo bench --bench pipeline --features composite,swizzle

# Pipeline benchmark (markdown output)
bench-pipeline-md:
    cargo bench --bench pipeline --features composite,swizzle -- --format=md

# Butteraugli quality-vs-size analysis for JPEG encoders
bench-quality:
    cargo run --example jpeg_quality --release --features composite,swizzle

# Run all tests
test:
    cargo test --features composite,swizzle

# Run clippy
clippy:
    cargo clippy --features composite,swizzle

# Format code
fmt:
    cargo fmt

# Full CI check (fmt + clippy + test)
ci: fmt clippy test
