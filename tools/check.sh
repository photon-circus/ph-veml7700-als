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
for target in \
    thumbv6m-none-eabi \
    thumbv7em-none-eabihf \
    thumbv8m.main-none-eabihf \
    riscv32imc-unknown-none-elf \
    riscv32imac-unknown-none-elf
do
    cargo check -p ph-veml7700-als --target "$target" --no-default-features
done
cargo deny check
cargo package -p ph-veml7700-als --list --allow-dirty
cargo run --manifest-path tools/ph-hil-build/Cargo.toml -- --mock
test -f target/ph-hil/veml7700-harness.manifest.toml
