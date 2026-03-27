use std::collections::BTreeSet;
use std::io::Error as IOError;
use std::process::{Command, Output, Stdio};

use crate::branches::Branches;
use crate::error::Error;
use crate::options::Options;

fn command_to_string(args: &[&str]) -> String {
    args.join(" ")
}

pub fn run_command_with_no_output(args: &[&str]) -> Result<(), Error> {
    let output = Command::new(args[0])
        .args(&args[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|source| Error::CommandExecution {
            command: command_to_string(args),
            source,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::CommandFailed {
            command: command_to_string(args),
            code: output.status.code(),
            stderr,
        });
    }

    Ok(())
}

pub fn output(args: &[&str]) -> Result<String, Error> {
    let result = run_command(args)?;
    String::from_utf8(result.stdout)
        .map(|stdout| stdout.trim().to_owned())
        .map_err(|source| Error::CommandOutputEncoding {
            command: command_to_string(args),
            source,
        })
}

pub fn run_command(args: &[&str]) -> Result<Output, Error> {
    run_command_with_result(args).map_err(|source| Error::CommandExecution {
        command: command_to_string(args),
        source,
    })
}

pub fn run_command_with_result(args: &[&str]) -> Result<Output, IOError> {
    Command::new(args[0]).args(&args[1..]).output()
}

pub fn validate_git_installation() -> Result<(), Error> {
    match Command::new("git").output() {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::GitInstallation),
    }
}

pub fn delete_local_branches(branches: &Branches) -> Result<String, Error> {
    // https://git-scm.com/docs/git-branch
    // With a -d or -D option, <branchname> will be deleted. You may specify more than one branch
    // for deletion.
    //
    // So we can work without xargs.
    if branches.vec.is_empty() {
        Ok(String::default())
    } else {
        let delete_branches_args =
            branches
                .vec
                .iter()
                .fold(vec!["git", "branch", "-D"], |mut acc, b| {
                    acc.push(b);
                    acc
                });
        let delete_branches_cmd = run_command(&delete_branches_args)?;
        String::from_utf8(delete_branches_cmd.stdout).map_err(|source| {
            Error::CommandOutputEncoding {
                command: command_to_string(&delete_branches_args),
                source,
            }
        })
    }
}

pub fn delete_remote_branches(branches: &Branches, options: &Options) -> Result<String, Error> {
    let remote_branches_cmd = run_command(&["git", "branch", "-r"])?;

    let s = String::from_utf8(remote_branches_cmd.stdout).map_err(|source| {
        Error::CommandOutputEncoding {
            command: "git branch -r".to_owned(),
            source,
        }
    })?;
    let all_remote_branches = s.split('\n').collect::<Vec<&str>>();
    let origin_for_trim = &format!("{}/", &options.remote)[..];
    let b_tree_remotes = all_remote_branches
        .iter()
        .map(|b| b.trim().trim_start_matches(origin_for_trim).to_owned())
        .collect::<BTreeSet<String>>();

    let mut b_tree_branches = BTreeSet::new();

    for branch in branches.vec.clone() {
        b_tree_branches.insert(branch);
    }

    let intersection: Vec<_> = b_tree_remotes
        .intersection(&b_tree_branches)
        .cloned()
        .collect();

    let stderr = if intersection.is_empty() {
        String::default()
    } else {
        let delete_branches_args = intersection.iter().fold(
            vec!["git", "push", &options.remote, "--delete"],
            |mut acc, b| {
                acc.push(b);
                acc
            },
        );
        let delete_remote_branches_cmd = run_command(&delete_branches_args)?;
        String::from_utf8(delete_remote_branches_cmd.stderr).map_err(|source| {
            Error::CommandOutputEncoding {
                command: command_to_string(&delete_branches_args),
                source,
            }
        })?
    };

    // Everything is written to stderr, so we need to process that
    let split = stderr.split('\n');
    let vec: Vec<&str> = split.collect();
    let mut output = vec![];
    for s in vec {
        if s.contains("error: unable to delete '") {
            let branch = s
                .trim_start_matches("error: unable to delete '")
                .trim_end_matches("': remote ref does not exist");

            output.push(branch.to_owned() + " was already deleted in the remote.");
        } else if s.contains(" - [deleted]") {
            output.push(s.to_owned());
        }
    }

    Ok(output.join("\n"))
}

#[cfg(test)]
mod test {

    use crate::error::Error;
    use regex::Regex;

    use super::{run_command, run_command_with_no_output};

    // `spawn_piped` was removed so this test is somewhat outdated.
    // It now tests the match operation for which `grep` was used before.
    #[test]
    fn test_spawn_piped() {
        let echo = Regex::new("foo\n").unwrap();
        assert_eq!(
            echo.captures_iter("foo\nbar\nbaz")
                .fold(String::new(), |mut acc, e| {
                    acc.push_str(&e[0]);
                    acc
                }),
            "foo\n"
        );
    }

    #[test]
    fn test_run_command_returns_typed_error_with_command_context() {
        let err = run_command(&["git-clean-command-that-does-not-exist", "--version"]).unwrap_err();

        match err {
            Error::CommandExecution { command, .. } => {
                assert_eq!(command, "git-clean-command-that-does-not-exist --version")
            }
            other => panic!("Expected CommandExecution, found {:?}", other),
        }
    }

    #[test]
    fn test_run_command_with_no_output_returns_typed_error_with_command_context() {
        let err =
            run_command_with_no_output(&["git-clean-command-that-does-not-exist", "--version"])
                .unwrap_err();

        match err {
            Error::CommandExecution { command, .. } => {
                assert_eq!(command, "git-clean-command-that-does-not-exist --version")
            }
            other => panic!("Expected CommandExecution, found {:?}", other),
        }
    }

    #[test]
    fn test_run_command_with_no_output_returns_error_on_non_zero_exit() {
        let err = run_command_with_no_output(&["git", "this-command-does-not-exist"]).unwrap_err();

        match err {
            Error::CommandFailed { command, .. } => {
                assert_eq!(command, "git this-command-does-not-exist")
            }
            other => panic!("Expected CommandFailed, found {:?}", other),
        }
    }
}
