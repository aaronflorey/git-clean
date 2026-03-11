#![deny(warnings)]

pub mod cli;

use clap::ArgMatches;

mod branches;
use crate::branches::Branches;

mod commands;
pub use commands::validate_git_installation;

mod config;

mod error;
use crate::error::Error;

mod options;
use crate::options::Options;

pub fn run(matches: &ArgMatches) -> Result<(), error::Error> {
    validate_git_installation()?;

    let current_directory = std::env::current_dir()?;
    let resolved = Options::with_repo_config(matches, &current_directory)?;
    let options = resolved.options;
    if resolved.used_repo_config {
        println!("Using repository config (after CLI overrides):");
        println!("{}", options.describe_effective());
    }
    Options::save_cli_flags(matches, &current_directory)?;
    options.validate()?;

    let branches = Branches::merged(&options);

    if branches.string.is_empty() {
        println!("No branches to delete, you're clean!");
        return Ok(());
    }

    if !matches.get_flag("yes") {
        branches.print_warning_and_prompt(&options.delete_mode)?;
    }

    let msg = branches.delete(&options);
    println!("\n{}", msg);

    Ok(())
}

pub fn print_and_exit(error: &Error) {
    println!("{}", error);
    std::process::exit(1);
}
