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

mod ui;
use crate::ui::Ui;

pub fn run(matches: &ArgMatches) -> Result<(), error::Error> {
    validate_git_installation()?;

    let current_directory = std::env::current_dir()?;
    let resolved = Options::with_repo_config(matches, &current_directory)?;
    let options = resolved.options;
    let ui = Ui::new(options.color_mode);
    if resolved.used_repo_config {
        println!("{}", ui.section("Effective settings"));
        println!("{}", ui.key_value("remote", &options.remote));
        println!("{}", ui.key_value("base branch", &options.base_branch));
        println!(
            "{}",
            ui.key_value("delete mode", options.delete_mode.as_str())
        );
        println!(
            "{}",
            ui.key_value(
                "delete unpushed branches",
                &options.delete_unpushed_branches.to_string()
            )
        );
        println!(
            "{}",
            ui.key_value("squash detection", &options.squashes.to_string())
        );
        println!(
            "{}",
            ui.key_value(
                "ignored branches",
                &format!("{:?}", options.ignored_branches)
            )
        );
        println!("{}", ui.key_value("color", options.color_mode.describe()));
    }
    Options::save_cli_flags(matches, &current_directory)?;
    options.validate()?;

    let branches = Branches::merged(&options)?;
    let target_count = branches.target_count(&options)?;

    if target_count == 0 {
        println!("{}", ui.success("No branches to delete, you're clean!"));
        return Ok(());
    }

    println!("{}", ui.section("Delete plan"));
    println!("{}", ui.key_value("mode", options.delete_mode.as_str()));
    println!(
        "{}",
        ui.key_value("branches matched", &target_count.to_string())
    );

    if !matches.get_flag("yes") {
        branches.print_warning_and_prompt(&options, &ui)?;
    }

    let msg = branches.delete(&options)?;
    println!("{}", ui.section("Delete result"));
    println!("{}", msg);

    Ok(())
}

pub fn print_and_exit(error: &Error) {
    println!("{}", error);
    std::process::exit(1);
}
