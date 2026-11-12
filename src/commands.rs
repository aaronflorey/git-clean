use std::collections::HashSet;
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
    let output = run_command_with_result(args).map_err(|source| Error::CommandExecution {
        command: command_to_string(args),
        source,
    })?;

    if !output.status.success() {
        return Err(Error::CommandFailed {
            command: command_to_string(args),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    Ok(output)
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
        return Ok(String::default());
    }

    let mut rows = Vec::with_capacity(branches.worktrees.len() + 1);
    for path in &branches.worktrees {
        run_command(&["git", "worktree", "remove", path])?;
        rows.push(format!("Removed worktree {path}"));
    }

    let delete_branches_args =
        branches
            .vec
            .iter()
            .fold(vec!["git", "branch", "-D"], |mut acc, b| {
                acc.push(b);
                acc
            });
    rows.push(output(&delete_branches_args)?);
    Ok(rows.join("\n"))
}

/// Names of the branches on `remote` as of the last fetch, without the `<remote>/` prefix.
pub fn remote_branch_names(remote: &str) -> Result<HashSet<String>, Error> {
    let prefix = format!("refs/remotes/{remote}/");
    let refs = output(&[
        "git",
        "for-each-ref",
        "--format=%(refname)",
        &format!("refs/remotes/{remote}"),
    ])?;

    Ok(refs
        .lines()
        .filter_map(|refname| refname.strip_prefix(&prefix))
        // `<remote>/HEAD` is a symbolic ref to the default branch, not a branch of its own.
        .filter(|name| *name != "HEAD")
        .map(str::to_owned)
        .collect())
}

pub fn delete_remote_branches(branches: &Branches, options: &Options) -> Result<String, Error> {
    let remote_branches = remote_branch_names(&options.remote)?;
    let targets: Vec<&str> = branches
        .vec
        .iter()
        .filter(|branch| remote_branches.contains(*branch))
        .map(String::as_str)
        .collect();

    if targets.is_empty() {
        return Ok(String::default());
    }

    // git exits non-zero unless every ref was deleted, and that error carries its full output.
    let mut delete_branches_args = vec!["git", "push", &options.remote, "--delete"];
    delete_branches_args.extend(&targets);
    run_command(&delete_branches_args)?;

    Ok(targets
        .iter()
        .map(|branch| format!("Deleted remote branch {}/{}", options.remote, branch))
        .collect::<Vec<String>>()
        .join("\n"))
}

#[cfg(test)]
mod test {

    use crate::error::Error;

    use super::{run_command, run_command_with_no_output};

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
