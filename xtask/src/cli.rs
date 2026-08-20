use clap::{Parser, Subcommand};

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
}
