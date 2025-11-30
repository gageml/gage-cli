# Show this help
help:
    just -l

# Debug build
build:
    cargo build

# Release build
release:
    maturin build --release

# Run tests
test: build
    cargo test
    groktest .
