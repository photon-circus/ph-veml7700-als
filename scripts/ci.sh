#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

driver_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/veml7700/Cargo.toml | head -n 1)
expected_driver_version=0.1.0-incubating.1
if [ "$driver_version" != "$expected_driver_version" ]; then
    echo "driver version must be $expected_driver_version: $driver_version" >&2
    exit 1
fi

cargo fmt --all -- --check
cargo test -p ph-veml7700-als --no-default-features
cargo test -p ph-veml7700-als-model --no-default-features
cargo check -p ph-veml7700-als --all-features
cargo check -p ph-veml7700-als-model
cargo clippy -p ph-veml7700-als --all-targets --all-features -- -D warnings
cargo clippy -p ph-veml7700-als-model --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p ph-veml7700-als --all-features --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc -p ph-veml7700-als-model --no-deps
cargo test -p ph-veml7700-als --doc
cargo test -p ph-veml7700-als-model --doc

for target in \
    thumbv6m-none-eabi \
    thumbv7em-none-eabihf \
    thumbv8m.main-none-eabihf \
    riscv32imc-unknown-none-elf \
    riscv32imac-unknown-none-elf
do
    cargo check -p ph-veml7700-als --target "$target" --no-default-features
    cargo check -p ph-veml7700-als-model --target "$target"
done

cargo deny check -D warnings

# Pin packaging to the repository target directory. Cargo excludes only that
# path from workspace member discovery, so an extracted package anywhere else
# inside the repository cannot be tested.
package_target_dir=$repo_root/target
package_dir=$package_target_dir/package
rm -rf "$package_dir"
cargo package -p ph-veml7700-als --locked --allow-dirty --target-dir "$package_target_dir" --list
cargo package -p ph-veml7700-als --locked --allow-dirty --target-dir "$package_target_dir"
cargo test --manifest-path "$package_dir"/ph-veml7700-als-*/Cargo.toml
