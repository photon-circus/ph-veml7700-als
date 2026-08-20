//! Canonical repository gate. Policy lives in `data/*.ron`; this crate runs it.

mod cargo_cmd;
mod checks;
mod cli;
mod config;
mod engine;
mod evidence;
mod git;
mod report;

use std::process::ExitCode;

use anyhow::Error;
use clap::Parser;

use cli::{Command, Xtask};

/// Parse argv and run the selected xtask command.
pub fn run() -> ExitCode {
    let xtask = Xtask::parse();
    match execute(xtask) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(&err);
            ExitCode::FAILURE
        }
    }
}

fn execute(xtask: Xtask) -> anyhow::Result<()> {
    match xtask.command {
        Command::Ci { profile, only } => engine::run_ci(profile.as_deref(), only.as_deref()),
    }
}

fn print_error(err: &Error) {
    eprintln!("{err:#}");
}
