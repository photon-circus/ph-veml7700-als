#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

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

cargo deny check -D warnings
cargo package -p ph-veml7700-als --locked --allow-dirty --list
cargo package -p ph-veml7700-als --locked --allow-dirty
