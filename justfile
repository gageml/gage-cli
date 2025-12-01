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

# Run example tests
test-examples: _gage_inspect_repo
    GAGE_INSPECT=../gage-inspect groktest tests/examples/*.md

_gage_inspect_repo:
    #!/bin/env sh
    if ! test -e ../gage-inspect; then
        echo "Missing ../gage-inspect - clone the repo and try again"
        exit 1
    fi

# Publish a release
publish version:
    uv publish target/wheels/gage_cli-{{version}}-*.whl
