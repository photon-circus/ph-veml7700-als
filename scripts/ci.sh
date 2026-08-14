#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

# `full` is the authoritative local gate. `bounded` is the subset hosted CI
# runs: it drops the checks that need an extra binary or substantial runner
# time, and reports each of them as an explicit skip. A skipped check is not a
# passed check, so a green bounded run never stands in for a green full run.
ci_profile=${CI_PROFILE:-full}
case "$ci_profile" in
    full | bounded) ;;
    *)
        echo "CI_PROFILE must be 'full' or 'bounded': $ci_profile" >&2
        exit 1
        ;;
esac

step_number=0
skipped=0
current_step="start"

printf '[ci] profile: %s\n' "$ci_profile"

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

skip() {
    skipped=$((skipped + 1))
    printf '        SKIP: %s\n' "$1"
}

step "verify the Incubating candidate version"
driver_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/veml7700/Cargo.toml | head -n 1)
expected_driver_version=0.1.0-incubating.1
if [ "$driver_version" != "$expected_driver_version" ]; then
    echo "driver version must be $expected_driver_version: $driver_version" >&2
    exit 1
fi

# `.gitignore` keeps vendor documents out by default, but `git add -f` and any
# previously tracked file bypass it. This check is what actually enforces the
# untracked claim in `docs/vendor/README.md` and I-26.
step "vendor documents remain untracked"
if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    tracked_vendor=$(git ls-files docs/vendor | grep -v '^docs/vendor/README\.md$' || true)
    if [ -n "$tracked_vendor" ]; then
        printf 'vendor documents must not be tracked:\n%s\n' "$tracked_vendor" >&2
        exit 1
    fi
else
    skip "no Git work tree, so tracked vendor documents cannot be checked"
fi

# The packaged README and crate documentation are what a consumer reads. They
# drifted apart once already; this keeps the required disclosure identical.
step "status disclosures agree across README, packaged README, and lib.rs"
disclosure() {
    sed -n '/\*\*Lifecycle:\*\*/,/hardware qualification\./p' "$1" \
        | sed -e 's|^[[:space:]]*//!||' -e 's|^[[:space:]]*>||' \
        | tr '\n' ' ' | tr -s '[:space:]' ' ' | sed -e 's|^ ||' -e 's| $||'
}
root_disclosure=$(disclosure README.md)
if [ -z "$root_disclosure" ]; then
    echo "no status disclosure found in README.md" >&2
    exit 1
fi
for disclosure_file in crates/veml7700/README.md crates/veml7700/src/lib.rs; do
    other_disclosure=$(disclosure "$disclosure_file")
    if [ "$root_disclosure" != "$other_disclosure" ]; then
        printf '%s disclosure differs from README.md\n  README.md: %s\n  %s: %s\n' \
            "$disclosure_file" "$root_disclosure" "$disclosure_file" "$other_disclosure" >&2
        exit 1
    fi
done

# Only the `lib.rs` copy is compiled; a README fence is inert. Without this the
# packaged README could render an example that no longer builds.
step "the usage example agrees between the packaged README and lib.rs"
example() {
    sed -n '/```rust,no_run/,/^\(\/\/! \)\?```$/p' "$1" \
        | sed -e 's|^//! \{0,1\}||' -e 's|^//!$||'
}
lib_example=$(example crates/veml7700/src/lib.rs)
if [ -z "$lib_example" ]; then
    echo "no compiled usage example found in crates/veml7700/src/lib.rs" >&2
    exit 1
fi
if [ "$lib_example" != "$(example crates/veml7700/README.md)" ]; then
    echo "the packaged README usage example differs from the lib.rs doctest" >&2
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
if [ "$ci_profile" = bounded ]; then
    bare_metal_targets="thumbv7em-none-eabihf"
    skip "four of the five supported targets; the full gate compiles all five"
else
    bare_metal_targets="thumbv6m-none-eabi
thumbv7em-none-eabihf
thumbv8m.main-none-eabihf
riscv32imc-unknown-none-elf
riscv32imac-unknown-none-elf"
fi
for target in $bare_metal_targets; do
    cargo check -p ph-veml7700-als --target "$target" --no-default-features
    cargo check -p ph-veml7700-als-model --target "$target"
done

step "dependency advisory and license policy"
if [ "$ci_profile" = bounded ]; then
    skip "cargo-deny is not provisioned for bounded runs; the full gate runs it"
else
    cargo deny check -D warnings
fi

# Pin packaging to the repository target directory. Cargo excludes only that
# path from workspace member discovery, so an extracted package anywhere else
# inside the repository cannot be tested.
step "package construction, inspection, and unpacked test"
if [ "$ci_profile" = bounded ]; then
    skip "packaging belongs to the release gate and runs locally"
else
    package_target_dir=$repo_root/target
    package_dir=$package_target_dir/package
    rm -rf "$package_dir"
    cargo package -p ph-veml7700-als --locked --allow-dirty --target-dir "$package_target_dir" --list
    cargo package -p ph-veml7700-als --locked --allow-dirty --target-dir "$package_target_dir"
    cargo test --manifest-path "$package_dir"/ph-veml7700-als-*/Cargo.toml
fi

trap - EXIT
printf '\n[ci] PASS (%s): %s steps, %s skipped.\n' "$ci_profile" "$step_number" "$skipped"
if [ "$ci_profile" = bounded ]; then
    printf '[ci] This is a partial gate. It covers only part of the release gate;\n'
    printf '[ci] the full local run remains authoritative.\n'
fi
printf '[ci] This gate establishes the implemented host boundary only. It does\n'
printf '[ci] not establish physical-device or calibrated-optical behavior.\n'
