#!/usr/bin/env bash
set -euo pipefail
python3 tools/validate-pack.py
python3 -m unittest tools/test_validate_pack.py
cargo fmt --all -- --check
cargo test -p ph-veml7700-als --no-default-features
cargo check -p ph-veml7700-als --all-features
cargo clippy -p ph-veml7700-als --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p ph-veml7700-als --all-features --no-deps
cargo test -p ph-veml7700-als --doc
cargo deny check
cargo package -p ph-veml7700-als --list --allow-dirty
