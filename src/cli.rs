use clap::{Arg, ArgAction, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn build_cli() -> Command {
    Command::new("git-clean")
        .version(VERSION)
        .about("A tool for cleaning old git branches.")
        .arg(
            Arg::new("locals")
                .short('l')
                .long("locals")
                .help("Only delete local branches")
                .conflicts_with("remotes")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("remotes")
                .short('r')
                .long("remotes")
                .help("Only delete remote branches")
                .conflicts_with("locals")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .help("Skip the check for deleting branches")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("squashes")
                .short('s')
                .long("squashes")
                .help("Check for squashes by finding branches incompatible with main")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("delete-unpushed-branches")
                .short('d')
                .long("delete-unpushed-branches")
                .help("Delete any local branch that is not present on the remote. Use this to speed up the checks if such branches should always be considered as merged")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("remote")
                .short('R')
                .long("remote")
                .help("Changes the git remote used (default is origin)")
                .num_args(1),
        )
        .arg(
            Arg::new("branch")
                .short('b')
                .long("branch")
                .help("Changes the base for merged branches (default is main)")
                .num_args(1),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .help("Ignore given branch (repeat option for multiple branches)")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("save-config")
                .long("save-config")
                .help("Save the command line flags used for this repository into ~/.config/git-clean/config.toml")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ignore-config")
                .long("ignore-config")
                .help("Ignore per-repository config values for this run")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .help("Control color output (auto, always, never)")
                .value_parser(["auto", "always", "never"])
                .num_args(1),
        )
}
