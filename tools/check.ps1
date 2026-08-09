$ErrorActionPreference = "Stop"
python tools/validate-pack.py
cargo fmt --all -- --check
cargo test -p ph-veml7700-als --no-default-features
cargo check -p ph-veml7700-als --all-features
cargo clippy -p ph-veml7700-als --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc -p ph-veml7700-als --all-features --no-deps
cargo test -p ph-veml7700-als --doc
cargo deny check
cargo package -p ph-veml7700-als --list
cargo publish -p ph-veml7700-als --dry-run
