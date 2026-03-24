use crate::commands::{output, run_command};
use crate::config::{self, RepoConfig};
use crate::error::Error;
use clap::parser::ValueSource;
use clap::ArgMatches;
use std::path::Path;

const DEFAULT_REMOTE: &str = "origin";
const DEFAULT_BRANCH: &str = "main";

#[derive(Debug, Clone)]
pub enum DeleteMode {
    Local,
    Remote,
    Both,
}

pub use self::DeleteMode::*;

#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

pub use self::ColorMode::*;

impl ColorMode {
    fn from_cli(value: &str) -> ColorMode {
        match value {
            "always" => Always,
            "never" => Never,
            _ => Auto,
        }
    }

    pub fn describe(&self) -> &'static str {
        match self {
            Auto => "auto",
            Always => "always",
            Never => "never",
        }
    }
}

impl DeleteMode {
    pub fn new(opts: &ArgMatches) -> DeleteMode {
        if opts.get_flag("locals") {
            Local
        } else if opts.get_flag("remotes") {
            Remote
        } else {
            Both
        }
    }

    pub fn warning_message(&self) -> String {
        let source = match *self {
            Local => "locally:",
            Remote => "remotely:",
            Both => "locally and remotely:",
        };
        format!("The following branches will be deleted {}", source)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Local => "local",
            Remote => "remote",
            Both => "local + remote",
        }
    }
}

pub struct Options {
    pub remote: String,
    pub base_branch: String,
    pub squashes: bool,
    pub delete_unpushed_branches: bool,
    pub ignored_branches: Vec<String>,
    pub delete_mode: DeleteMode,
    pub color_mode: ColorMode,
}

pub struct ResolvedOptions {
    pub options: Options,
    pub used_repo_config: bool,
}

impl Options {
    #[cfg(test)]
    pub fn new(opts: &ArgMatches) -> Options {
        let mut options = Self::default();
        options.apply_flag_overrides(opts);
        options
    }

    pub fn with_repo_config(opts: &ArgMatches, repo_path: &Path) -> Result<ResolvedOptions, Error> {
        let mut options = Self::default();
        let mut used_repo_config = false;

        if !opts.get_flag("ignore-config") {
            if let Some(repo_config) = config::load_repo_config(repo_path)? {
                repo_config.apply_to_options(&mut options);
                used_repo_config = true;
            }
        }

        options.apply_flag_overrides(opts);
        Ok(ResolvedOptions {
            options,
            used_repo_config,
        })
    }

    pub fn save_cli_flags(opts: &ArgMatches, repo_path: &Path) -> Result<bool, Error> {
        if !opts.get_flag("save-config") {
            return Ok(false);
        }

        let patch = Self::config_patch_from_matches(opts);
        if patch.is_empty() {
            return Ok(false);
        }

        config::save_repo_config(repo_path, &patch)?;
        Ok(true)
    }

    fn default() -> Options {
        Options {
            remote: DEFAULT_REMOTE.to_owned(),
            base_branch: DEFAULT_BRANCH.to_owned(),
            ignored_branches: Vec::new(),
            squashes: false,
            delete_unpushed_branches: false,
            delete_mode: Both,
            color_mode: Auto,
        }
    }

    fn apply_flag_overrides(&mut self, opts: &ArgMatches) {
        if let Some(remote) = opts.get_one::<String>("remote") {
            self.remote = remote.clone();
        }

        if let Some(base_branch) = opts.get_one::<String>("branch") {
            self.base_branch = base_branch.clone();
        }

        if let Some(ignored) = opts.get_many::<String>("ignore") {
            self.ignored_branches = ignored.cloned().collect();
        }

        if opts.get_flag("squashes") {
            self.squashes = true;
        }

        if opts.get_flag("delete-unpushed-branches") {
            self.delete_unpushed_branches = true;
        }

        if opts.value_source("locals") == Some(ValueSource::CommandLine)
            || opts.value_source("remotes") == Some(ValueSource::CommandLine)
        {
            self.delete_mode = DeleteMode::new(opts);
        }

        if let Some(color) = opts.get_one::<String>("color") {
            self.color_mode = ColorMode::from_cli(color);
        }
    }

    fn config_patch_from_matches(opts: &ArgMatches) -> RepoConfig {
        let mut patch = RepoConfig::default();

        if let Some(remote) = opts.get_one::<String>("remote") {
            patch.remote = Some(remote.clone());
        }

        if let Some(branch) = opts.get_one::<String>("branch") {
            patch.base_branch = Some(branch.clone());
        }

        if opts.get_flag("squashes") {
            patch.squashes = Some(true);
        }

        if opts.get_flag("delete-unpushed-branches") {
            patch.delete_unpushed_branches = Some(true);
        }

        if let Some(ignored) = opts.get_many::<String>("ignore") {
            patch.ignored_branches = Some(ignored.cloned().collect());
        }

        if opts.value_source("locals") == Some(ValueSource::CommandLine)
            || opts.value_source("remotes") == Some(ValueSource::CommandLine)
        {
            patch.delete_mode = Some(config::delete_mode_to_config(&DeleteMode::new(opts)));
        }

        if let Some(color) = opts.get_one::<String>("color") {
            patch.color_mode = Some(config::color_mode_to_config(&ColorMode::from_cli(color)));
        }

        patch
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.validate_base_branch()?;
        self.validate_remote()?;
        Ok(())
    }

    fn validate_base_branch(&self) -> Result<(), Error> {
        let current_branch = output(&["git", "rev-parse", "--abbrev-ref", "HEAD"])?;

        if current_branch != self.base_branch {
            return Err(Error::CurrentBranchInvalid);
        };

        Ok(())
    }

    fn validate_remote(&self) -> Result<(), Error> {
        let remotes = run_command(&["git", "remote"])?;
        let remotes_output =
            String::from_utf8(remotes.stdout).map_err(|source| Error::CommandOutputEncoding {
                command: "git remote".to_owned(),
                source,
            })?;

        if !remotes_output
            .lines()
            .any(|remote| remote.trim() == self.remote)
        {
            return Err(Error::InvalidRemote);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{ColorMode, DeleteMode, Options};
    use crate::cli;

    // Helpers
    fn parse_args(args: Vec<&str>) -> clap::ArgMatches {
        cli::build_cli().get_matches_from(args)
    }

    // DeleteMode tests
    #[test]
    fn test_delete_mode_new() {
        let matches = parse_args(vec!["git-clean", "-l"]);

        match DeleteMode::new(&matches) {
            DeleteMode::Local => (),
            other => panic!("Expected a DeleteMode::Local, but found: {:?}", other),
        };

        let matches = parse_args(vec!["git-clean", "-r"]);

        match DeleteMode::new(&matches) {
            DeleteMode::Remote => (),
            other => panic!("Expected a DeleteMode::Remote, but found: {:?}", other),
        };

        let matches = parse_args(vec!["git-clean"]);

        match DeleteMode::new(&matches) {
            DeleteMode::Both => (),
            other => panic!("Expected a DeleteMode::Both, but found: {:?}", other),
        };
    }

    #[test]
    fn test_delete_mode_warning_message() {
        assert_eq!(
            "The following branches will be deleted locally:",
            DeleteMode::Local.warning_message()
        );
        assert_eq!(
            "The following branches will be deleted remotely:",
            DeleteMode::Remote.warning_message()
        );
        assert_eq!(
            "The following branches will be deleted locally and remotely:",
            DeleteMode::Both.warning_message()
        );
    }

    #[test]
    fn test_color_mode_defaults_to_auto() {
        let matches = parse_args(vec!["git-clean"]);
        let options = Options::new(&matches);

        match options.color_mode {
            ColorMode::Auto => (),
            other => panic!("Expected ColorMode::Auto, but found: {:?}", other),
        };
    }

    #[test]
    fn test_color_mode_override_from_cli() {
        let matches = parse_args(vec!["git-clean", "--color", "never"]);
        let options = Options::new(&matches);

        match options.color_mode {
            ColorMode::Never => (),
            other => panic!("Expected ColorMode::Never, but found: {:?}", other),
        };
    }

    // Options tests
    #[test]
    fn test_git_options_new() {
        let matches = parse_args(vec!["git-clean"]);
        let git_options = Options::new(&matches);

        assert_eq!("main".to_owned(), git_options.base_branch);
        assert_eq!("origin".to_owned(), git_options.remote);

        let matches = parse_args(vec!["git-clean", "-b", "stable"]);
        let git_options = Options::new(&matches);

        assert_eq!("stable".to_owned(), git_options.base_branch);
        assert_eq!("origin".to_owned(), git_options.remote);

        let matches = parse_args(vec!["git-clean", "-R", "upstream"]);
        let git_options = Options::new(&matches);

        assert_eq!("main".to_owned(), git_options.base_branch);
        assert_eq!("upstream".to_owned(), git_options.remote);
        assert!(!git_options.squashes);
        assert!(!git_options.delete_unpushed_branches);

        let matches = parse_args(vec![
            "git-clean",
            "-R",
            "upstream",
            "--squashes",
            "--delete-unpushed-branches",
        ]);
        let git_options = Options::new(&matches);

        assert!(git_options.squashes);
        assert!(git_options.delete_unpushed_branches);

        let matches = parse_args(vec![
            "git-clean",
            "-i",
            "branch1",
            "-i",
            "branch2",
            "-i",
            "branch3",
        ]);
        let git_options = Options::new(&matches);

        assert_eq!(
            git_options.ignored_branches,
            vec!["branch1", "branch2", "branch3"]
        );
    }

    #[test]
    fn test_save_without_flags_is_noop() {
        let matches = parse_args(vec!["git-clean", "--save-config"]);
        let temp = tempfile::TempDir::new().unwrap();
        let did_save = Options::save_cli_flags(&matches, temp.path()).unwrap();
        assert!(!did_save);
    }
}
