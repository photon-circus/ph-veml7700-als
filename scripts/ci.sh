#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

# `full` is the authoritative local gate. `bounded` is the subset hosted CI
# runs: it drops the checks that need an extra binary or substantial runner
# time, and reports each of them as an explicit skip. A skipped check is not a
# passed check, so a green bounded run never stands in for a green full run.
#
# `release` is `full` plus artifact identity. It refuses a dirty worktree,
# packages without `--allow-dirty`, and records the exact source-to-archive
# relationship as evidence. It performs no registry action of any kind: no
# publish, no tag, no release, no credential use. Ordinary development keeps
# using `full`, which still packages permissively so a work-in-progress tree
# stays checkable.
ci_profile=${CI_PROFILE:-full}
case "$ci_profile" in
    full | bounded | release) ;;
    *)
        echo "CI_PROFILE must be 'full', 'bounded', or 'release': $ci_profile" >&2
        exit 1
        ;;
esac

# The one supported `cargo-deny`. An unpinned advisory tool makes the
# authoritative gate non-reproducible: two contributors run the same command and
# get different results as the advisory database and the tool's own rules move.
expected_cargo_deny_version=0.20.2

evidence_dir=$repo_root/target/release-evidence
evidence_file=$evidence_dir/evidence.md
if [ "$ci_profile" = release ]; then
    rm -rf "$evidence_dir"
    mkdir -p "$evidence_dir"
    : > "$evidence_file"
fi

# Append a line to the release evidence record. A no-op outside release mode, so
# call sites do not need to branch.
evidence() {
    if [ "$ci_profile" = release ]; then
        printf '%s\n' "$1" >> "$evidence_file"
    fi
}

# There is no portable SHA-256 binary. `sha256sum` is GNU coreutils and is what
# Git Bash and most Linux distributions provide; stock macOS ships `shasum`
# instead and would otherwise fail with exit 127 after packaging, producing no
# evidence at all. Probe rather than assume, and fail with a useful message if
# none of the three is present.
sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$1" | awk '{print $NF}'
    else
        echo "release evidence needs a SHA-256 tool: sha256sum, shasum, or openssl" >&2
        return 1
    fi
}

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

# Release evidence is only meaningful if the archive can be traced to a commit.
# `cargo package` without `--allow-dirty` is the authoritative check on package
# contents later, but it runs last and reports awkwardly; this fails immediately
# with the offending paths, and covers documentation and tooling that never
# enter the archive yet still describe the release.
#
# Both halves matter. A modified tracked file means the archive does not match
# the commit it claims. An untracked, non-ignored file is worse: `cargo package`
# would sweep it into the archive, so the published crate would contain content
# that exists in no commit at all.
if [ "$ci_profile" = release ]; then
    step "release worktree is clean"
    if ! command -v git >/dev/null 2>&1 || ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "release mode requires a Git work tree to establish artifact identity" >&2
        exit 1
    fi
    release_dirt=$(git status --porcelain --untracked-files=all)
    if [ -n "$release_dirt" ]; then
        printf 'release mode requires a clean worktree; found:\n%s\n' "$release_dirt" >&2
        printf 'commit, stash, or remove these before producing release evidence.\n' >&2
        exit 1
    fi
    release_commit=$(git rev-parse HEAD)
    printf '        clean at %s\n' "$release_commit"
    evidence "# Release evidence"
    evidence ""
    evidence "Produced by \`CI_PROFILE=release scripts/ci.sh\`. No registry action was"
    evidence "performed: no publish, no tag, no release, no credential use."
    evidence ""
    evidence "## Source"
    evidence ""
    evidence "- Commit: \`$release_commit\`"
    evidence "- Worktree: clean, including untracked files"
fi

# Both workspace crates share one version, so drift between them fails here
# rather than surfacing at release assembly. The exact version is not repeated
# in this script: it is read back through Cargo, and only the lifecycle-matching
# prerelease identifier is asserted. A bump therefore edits one manifest line,
# not the gate.
#
# Read the resolved version through Cargo rather than parsing manifest text.
# Both crates inherit `version` from `[workspace.package]`, so no literal
# `version = "..."` line exists in either crate manifest and a text parser would
# silently read nothing at all -- which is worse than failing, because an empty
# version compares equal to an empty version and the drift check would pass.
# `cargo pkgid` reports the same resolved metadata as `cargo metadata` without
# requiring a JSON parser, keeping `jq` off the prerequisite list.
step "verify the Incubating candidate version"
package_version() {
    cargo pkgid -p "$1" | sed 's/.*@//'
}
expected_lifecycle=incubating
driver_version=$(package_version ph-veml7700-als)
model_version=$(package_version ph-veml7700-als-model)
if [ -z "$driver_version" ] || [ -z "$model_version" ]; then
    echo "could not resolve a package version through cargo pkgid" >&2
    exit 1
fi
if [ "$driver_version" != "$model_version" ]; then
    printf 'workspace crates must share one version: driver %s, model %s\n' \
        "$driver_version" "$model_version" >&2
    exit 1
fi
# Match the complete version, not a substring. A shell glob would accept
# `0.1.0-incubating.1foo` and `0.1.0-foo-incubating.2`, both of which are valid
# Cargo versions that are not the documented form.
if ! printf '%s\n' "$driver_version" \
    | grep -Eq "^[0-9]+\.[0-9]+\.[0-9]+-$expected_lifecycle\.[0-9]+$"; then
    printf 'version must be exactly X.Y.Z-%s.N: %s\n' \
        "$expected_lifecycle" "$driver_version" >&2
    exit 1
fi
printf '        candidate version %s\n' "$driver_version"
evidence "- Candidate version: \`$driver_version\` (both crates, inherited from \`[workspace.package]\`)"

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

# The coverage matrix is the packaged claim about what model conformance
# establishes. It names tests, so it rots the moment a test is renamed, removed,
# or added without updating it -- and a stale matrix is worse than none, because
# it reads as an exact claim. Two directions, both required:
#
#   1. every test the matrix names must exist, or the claim cites nothing;
#   2. every conformance test must appear in the matrix, or coverage exists that
#      the packaged claim does not disclose.
#
# The second direction is the one that matters for honesty. Only the first is
# obvious.
step "the conformance matrix matches the conformance tests"
conformance_source=crates/veml7700/tests/device_model.rs
matrix_source=crates/veml7700/README.md
actual_tests=$(grep -A1 '^#\[test\]' "$conformance_source" | sed -n 's/^fn \([a-z0-9_]*\).*/\1/p' | sort -u)
if [ -z "$actual_tests" ]; then
    echo "no conformance tests found in $conformance_source" >&2
    exit 1
fi
matrix_block=$(sed -n '/^## Model conformance coverage/,/^## Usage/p' "$matrix_source")
if [ -z "$matrix_block" ]; then
    echo "no coverage matrix found in $matrix_source" >&2
    exit 1
fi
missing_from_matrix=
for conformance_test in $actual_tests; do
    if ! printf '%s\n' "$matrix_block" | grep -qF "$conformance_test"; then
        missing_from_matrix="$missing_from_matrix $conformance_test"
    fi
done
if [ -n "$missing_from_matrix" ]; then
    printf 'conformance tests absent from the packaged coverage matrix:%s\n' \
        "$missing_from_matrix" >&2
    printf 'coverage that the packaged claim does not disclose is the failure mode this check exists for.\n' >&2
    exit 1
fi
# Only the final column of the "Covered" table names tests. Operation names are
# backticked too, and the "Not covered" table backticks them in prose, so a
# whole-block scan would treat `measure_once` as a missing test.
named_tests=$(printf '%s\n' "$matrix_block" \
    | sed -n '/^### Covered/,/^### Not covered/p' \
    | awk -F'|' 'NF >= 5 { print $(NF - 1) }' \
    | grep -oE '`[a-z0-9_]+`' | tr -d '`' | sort -u)
missing_from_tests=
for named_test in $named_tests; do
    if ! grep -qF "fn $named_test(" "$conformance_source"; then
        missing_from_tests="$missing_from_tests $named_test"
    fi
done
if [ -n "$missing_from_tests" ]; then
    printf 'coverage matrix names tests that do not exist:%s\n' "$missing_from_tests" >&2
    exit 1
fi
printf '        %s conformance tests, all disclosed\n' \
    "$(printf '%s\n' "$actual_tests" | wc -l | tr -d ' ')"

# The matrix must reach a consumer, so it lives in the packaged README and is
# mirrored into the crate documentation. Compare them rather than trusting that
# two edits happened.
step "the coverage matrix agrees between the packaged README and lib.rs"
lib_matrix=$(sed -n '/^\/\/! ## Model conformance coverage/,/^\/\/! # Status/p' crates/veml7700/src/lib.rs \
    | sed -e 's|^//! \{0,1\}||' -e 's|^//!$||' | sed '/^# Status/d')
readme_matrix=$(printf '%s\n' "$matrix_block" | sed '/^## Usage/d')
if [ "$(printf '%s\n' "$lib_matrix" | sed '/^$/d')" != "$(printf '%s\n' "$readme_matrix" | sed '/^$/d')" ]; then
    echo "the coverage matrix differs between crates/veml7700/README.md and lib.rs" >&2
    exit 1
fi

# Rustdoc validates intra-doc links but never looks at repository Markdown, so
# a renamed or deleted document breaks its inbound links silently. That matters
# most at release: the packaged README and the contract documents it points at
# are what a consumer follows.
#
# This check is deliberately local and offline. It resolves relative paths and
# `#heading` anchors within the repository and nothing else. External URLs are
# not fetched: a network check is non-deterministic, and a gate that fails
# because a vendor site is briefly down teaches contributors to ignore it. Keep
# any external link checking in a separate, bounded, non-blocking job.
step "local Markdown links resolve"
heading_anchors() {
    # GitHub's slug: drop the leading hashes, remove inline code, emphasis and
    # link syntax, lowercase, delete anything outside [a-z0-9 -], then map runs
    # of whitespace to single hyphens.
    #
    # Repeated headings get `-1`, `-2`, ... in document order, which is what
    # GitHub does and what a link to a later occurrence relies on. Without the
    # suffixes this check would reject a link that resolves correctly on GitHub,
    # and a gate that fails on valid input is worse than one that misses.
    sed -n 's/^#\{1,6\}[[:space:]]\{1,\}//p' "$1" \
        | sed -e 's/`//g' -e 's/\*\*//g' -e 's/\*//g' \
              -e 's/\[\([^]]*\)\]([^)]*)/\1/g' \
        | tr '[:upper:]' '[:lower:]' \
        | sed -e 's/[^a-z0-9 -]//g' -e 's/[[:space:]]\{1,\}/-/g' \
        | awk '{ if (seen[$0]++) print $0 "-" seen[$0] - 1; else print $0 }'
}
markdown_targets() {
    # Match `](target)`, tolerating the angle-bracket form. Targets containing
    # whitespace or parentheses are skipped rather than mis-parsed.
    grep -o '\](<\?[^)>"[:space:]]\{1,\}>\?)' "$1" 2>/dev/null \
        | sed -e 's/^](//' -e 's/)$//' -e 's/^<//' -e 's/>$//'
}
link_report=$(
    for markdown_file in $(git ls-files '*.md'); do
        markdown_dir=$(dirname "$markdown_file")
        markdown_targets "$markdown_file" | while IFS= read -r target; do
            case "$target" in
                http://* | https://* | mailto:* | '') continue ;;
            esac
            target_path=${target%%#*}
            target_anchor=${target#*#}
            [ "$target_anchor" = "$target" ] && target_anchor=
            if [ -z "$target_path" ]; then
                resolved=$markdown_file
            elif [ "$markdown_dir" = "." ]; then
                resolved=$target_path
            else
                resolved=$markdown_dir/$target_path
            fi
            # Collapse `foo/../` without realpath, which is not universally present.
            resolved=$(printf '%s\n' "$resolved" \
                | sed -e ':a' -e 's|[^/][^/]*/\.\./||; ta' -e 's|^\./||')
            if [ ! -e "$resolved" ]; then
                printf 'BROKEN  %s -> %s\n' "$markdown_file" "$target"
                continue
            fi
            if [ -n "$target_anchor" ] && [ "${resolved%.md}" != "$resolved" ]; then
                if ! heading_anchors "$resolved" | grep -qx "$target_anchor"; then
                    printf 'ANCHOR  %s -> %s\n' "$markdown_file" "$target"
                    continue
                fi
            fi
            printf 'ok\n'
        done
    done
)
link_failures=$(printf '%s\n' "$link_report" | grep -v '^ok$' | grep -v '^$' || true)
if [ -n "$link_failures" ]; then
    printf 'local Markdown links that do not resolve:\n%s\n' "$link_failures" >&2
    exit 1
fi
printf '        %s local links resolve\n' \
    "$(printf '%s\n' "$link_report" | grep -c '^ok$' || true)"

# A variant no code ever names is surface a caller can match and never reach.
# `Operation::Probe` was exactly that: `probe` reports through `ProbeError`.
# The enum declarations use bare variant names, so only real `Enum::Variant`
# paths match here.
step "every public error variant is named by some code path"
error_variants=$(awk '
    /^pub enum [A-Za-z]+/ { name = $3; sub(/<.*/, "", name); in_enum = 1; next }
    in_enum && /^\}/ { in_enum = 0; next }
    in_enum && /^    [A-Z]/ { v = $1; sub(/[^A-Za-z0-9_].*/, "", v); print name "::" v }
' crates/veml7700/src/error.rs)
if [ -z "$error_variants" ]; then
    echo "no public error variants found in crates/veml7700/src/error.rs" >&2
    exit 1
fi
unreachable_variants=
for error_variant in $error_variants; do
    if ! grep -rqF "$error_variant" crates --include=*.rs; then
        unreachable_variants="$unreachable_variants $error_variant"
    fi
done
if [ -n "$unreachable_variants" ]; then
    printf 'public error variants that no code names:%s\n' "$unreachable_variants" >&2
    exit 1
fi
printf '        %s public error variants, all reachable\n' \
    "$(printf '%s\n' "$error_variants" | wc -l | tr -d ' ')"

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
    # Assert the tool version before trusting its verdict. `cargo-deny` changes
    # its own lint set and defaults between releases, so an unpinned binary
    # makes this step's result depend on whatever the runner happens to have
    # installed. Refuse rather than warn: a silently different advisory result
    # is exactly the kind of irreproducibility release evidence must exclude.
    actual_cargo_deny_version=$(cargo deny --version | awk '{print $2}')
    if [ "$actual_cargo_deny_version" != "$expected_cargo_deny_version" ]; then
        printf 'cargo-deny %s is required, found %s\n' \
            "$expected_cargo_deny_version" "$actual_cargo_deny_version" >&2
        printf 'install it with: cargo install cargo-deny --version %s --locked\n' \
            "$expected_cargo_deny_version" >&2
        exit 1
    fi
    printf '        cargo-deny %s\n' "$actual_cargo_deny_version"
    cargo deny check -D warnings
    evidence "- \`cargo-deny\` $actual_cargo_deny_version: advisory, source, and licence policy pass"
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
    # `full` packages permissively so a work-in-progress tree stays checkable.
    # `release` drops `--allow-dirty`, which makes Cargo itself refuse to build
    # an archive from a tree that does not match its commit -- the authoritative
    # check on package contents, after the earlier fast one.
    if [ "$ci_profile" = release ]; then
        package_dirt_flag=
    else
        package_dirt_flag=--allow-dirty
    fi
    # shellcheck disable=SC2086
    cargo package -p ph-veml7700-als --locked $package_dirt_flag --target-dir "$package_target_dir" --list
    # shellcheck disable=SC2086
    cargo package -p ph-veml7700-als --locked $package_dirt_flag --target-dir "$package_target_dir"

    # Capture evidence from the archive before testing the unpacked copy.
    # `cargo test` builds inside the unpacked directory, so anything that walks
    # that tree afterwards reports the test's own `target/` output as package
    # content. Read the inventory out of the `.crate` itself: that is what will
    # actually be uploaded, and it cannot drift with local build state.
    if [ "$ci_profile" = release ]; then
        package_archive=$package_dir/ph-veml7700-als-$driver_version.crate
        package_unpacked=$package_dir/ph-veml7700-als-$driver_version
        if [ ! -f "$package_archive" ]; then
            printf 'expected prepublication archive not found: %s\n' "$package_archive" >&2
            exit 1
        fi
        package_sha=$(sha256_of "$package_archive")
        package_inventory=$(tar -tzf "$package_archive" \
            | sed "s|^ph-veml7700-als-$driver_version/||" \
            | grep -v '^$' | sort)
        package_file_count=$(printf '%s\n' "$package_inventory" | wc -l | tr -d ' ')
        printf '        archive %s\n' "$(basename "$package_archive")"
        printf '        sha256  %s\n' "$package_sha"
        printf '        %s files in the archive\n' "$package_file_count"

        evidence ""
        evidence "## Prepublication archive"
        evidence ""
        evidence "This is the archive Cargo builds from the tree above. It is **not** the"
        evidence "byte-identical artifact a later \`cargo publish\` uploads: publishing"
        evidence "repackages from the source tree. Publish only from this same unchanged"
        evidence "clean pinned tree, then download the registry archive and verify its"
        evidence "checksum and contents against this record."
        evidence ""
        evidence "- File: \`$(basename "$package_archive")\`"
        evidence "- SHA-256: \`$package_sha\`"
        evidence "- Files: $package_file_count"
        evidence ""
        evidence "### File inventory"
        evidence ""
        evidence "Read from the \`.crate\` archive, not from the unpacked directory: the"
        evidence "unpacked copy accumulates \`target/\` output once its tests run."
        evidence ""
        evidence '```text'
        printf '%s\n' "$package_inventory" >> "$evidence_file"
        evidence '```'
        evidence ""
        evidence "### Normalized manifest"
        evidence ""
        evidence '```toml'
        cat "$package_unpacked/Cargo.toml" >> "$evidence_file"
        evidence '```'
        evidence ""
        evidence "### VCS metadata"
        evidence ""
        evidence '```json'
        if [ -f "$package_unpacked/.cargo_vcs_info.json" ]; then
            cat "$package_unpacked/.cargo_vcs_info.json" >> "$evidence_file"
        else
            evidence '{ "error": "no .cargo_vcs_info.json in the archive" }'
        fi
        evidence '```'
        evidence ""
        evidence "## Test boundary"
        evidence ""
        evidence "Two different things were tested, and the distinction matters:"
        evidence ""
        evidence "- **Standalone package tests** ran against the unpacked archive above."
        evidence "  They establish that the published crate builds and passes its own tests"
        evidence "  in isolation."
        evidence "- **Driver-versus-model conformance** ran only in the repository."
        evidence "  \`tests/device_model.rs\` and the path-only \`ph-veml7700-als-model\`"
        evidence "  dev-dependency are excluded from the package on purpose, so the archive"
        evidence "  cannot rerun them. Model conformance is repository-level evidence and"
        evidence "  must never be cited as package-level evidence."
        evidence ""
        evidence "\`ph-veml7700-als-model\` is repository-only and unpublished."
    fi

    # Last, because it builds inside the unpacked directory. Everything recorded
    # above describes the archive as Cargo produced it.
    cargo test --manifest-path "$package_dir"/ph-veml7700-als-*/Cargo.toml
fi

if [ "$ci_profile" = release ]; then
    evidence ""
    evidence "## Not established"
    evidence ""
    evidence "This evidence covers artifact identity and the implemented host boundary."
    evidence "It establishes no physical-device, electrical, optical, or calibrated"
    evidence "behavior, and it is not a decision to publish."
fi

trap - EXIT
printf '\n[ci] PASS (%s): %s steps, %s skipped.\n' "$ci_profile" "$step_number" "$skipped"
if [ "$ci_profile" = bounded ]; then
    printf '[ci] This is a partial gate. It covers only part of the release gate;\n'
    printf '[ci] the full local run remains authoritative.\n'
fi
if [ "$ci_profile" = release ]; then
    printf '[ci] Release evidence: %s\n' "${evidence_file#"$repo_root"/}"
    printf '[ci] Commit %s, archive SHA-256 %s\n' "$release_commit" "$package_sha"
    printf '[ci] No registry action was performed: no publish, tag, release, or\n'
    printf '[ci] credential use. Publication remains a separate recorded decision.\n'
fi
printf '[ci] This gate establishes the implemented host boundary only. It does\n'
printf '[ci] not establish physical-device or calibrated-optical behavior.\n'
