use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "Repository automation for ph-veml7700-als")]
pub struct Xtask {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the canonical verification gate.
    Ci {
        /// `full` (default), `bounded`, or `release`.
        ///
        /// Unset resolves to `default_profile` in `gate.ron`.
        #[arg(long)]
        profile: Option<String>,
        /// Run a single step id from `gate.ron`.
        #[arg(long)]
        only: Option<String>,
    },
    /// Verify a `.crate` against the commit it declares.
    ///
    /// Compares every packaged source entry against the Git blob at that
    /// commit, so a difference in line endings is reported as such and only
    /// edited content fails.
    #[command(group(
        ArgGroup::new("source").required(true).args(["archive", "version"])
    ))]
    VerifyPackage {
        /// Verify a local `.crate`. Contacts nothing.
        #[arg(long, value_name = "PATH")]
        archive: Option<PathBuf>,
        /// Download this published version from crates.io and verify it.
        #[arg(long, value_name = "VERSION", conflicts_with = "archive")]
        version: Option<String>,
        /// Revision the archive must declare. Defaults to `v<version>` with
        /// `--version`; required with `--archive`, because an archive's own
        /// declaration establishes nothing on its own.
        #[arg(long, value_name = "REV", required_unless_present = "version")]
        rev: Option<String>,
        /// Expected archive SHA-256. With `--version` the sparse index
        /// supplies it; pass this to pin the value that was reviewed.
        #[arg(long, value_name = "HEX")]
        sha256: Option<String>,
        /// Where a downloaded archive is written.
        #[arg(long, value_name = "DIR", default_value = "target/verify-package")]
        download_dir: PathBuf,
    },
}
