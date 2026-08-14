#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

step_number=0
current_step="start"

report_outcome() {
    status=$?
    if [ "$status" -ne 0 ]; then
        printf '\n[ci] FAIL at step %s: %s\n' "$step_number" "$current_step" >&2
    fi
}
trap report_outcome EXIT

step() {
    step_number=$((step_number + 1))
    current_step=$1
    printf '\n[ci %02d] %s\n' "$step_number" "$current_step"
}

step "verify the Incubating candidate version"
driver_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/veml7700/Cargo.toml | head -n 1)
expected_driver_version=0.1.0-incubating.1
if [ "$driver_version" != "$expected_driver_version" ]; then
    echo "driver version must be $expected_driver_version: $driver_version" >&2
    exit 1
fi

step "formatting"
cargo fmt --all -- --check

step "host tests, including doctests, without default features"
cargo test -p ph-veml7700-als --no-default-features
cargo test -p ph-veml7700-als-model --no-default-features

step "feature-complete compilation"
cargo check -p ph-veml7700-als --all-features
cargo check -p ph-veml7700-als-model

step "lints with warnings denied"
cargo clippy -p ph-veml7700-als --all-targets --all-features -- -D warnings
cargo clippy -p ph-veml7700-als-model --all-targets -- -D warnings

step "documentation build"
RUSTDOCFLAGS="-D warnings" cargo doc -p ph-veml7700-als --all-features --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc -p ph-veml7700-als-model --no-deps

# The no-default-features run above already covers doctests for both crates.
# This run is the only one that exercises them with every feature enabled.
step "doctests with all features"
cargo test -p ph-veml7700-als --all-features --doc

step "supported bare-metal targets"
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

step "dependency advisory and license policy"
cargo deny check -D warnings

# Pin packaging to the repository target directory. Cargo excludes only that
# path from workspace member discovery, so an extracted package anywhere else
# inside the repository cannot be tested.
step "package construction, inspection, and unpacked test"
package_target_dir=$repo_root/target
package_dir=$package_target_dir/package
rm -rf "$package_dir"
cargo package -p ph-veml7700-als --locked --allow-dirty --target-dir "$package_target_dir" --list
cargo package -p ph-veml7700-als --locked --allow-dirty --target-dir "$package_target_dir"
cargo test --manifest-path "$package_dir"/ph-veml7700-als-*/Cargo.toml

trap - EXIT
printf '\n[ci] PASS: %s steps, 0 skipped.\n' "$step_number"
printf '[ci] This gate establishes the implemented host boundary only. It does\n'
printf '[ci] not establish physical-device or calibrated-optical behavior.\n'
