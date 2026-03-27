use crate::commands::*;
use crate::error::Error;
use crate::options::*;
use crate::ui::Ui;
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::io::{stdin, stdout, Write};

const PREVIEW_BRANCH_LIMIT: usize = 20;

#[derive(Debug)]
pub struct Branches {
    pub vec: Vec<String>,
}

impl Branches {
    pub fn new(branches: Vec<String>) -> Branches {
        Branches { vec: branches }
    }

    pub fn print_warning_and_prompt(&self, options: &Options, ui: &Ui) -> Result<(), Error> {
        let grouped = self.grouped_preview(options)?;

        println!("{}", ui.section("Delete preview"));
        println!("{}", ui.warning(&options.delete_mode.warning_message()));
        println!(
            "{}",
            ui.key_value("delete mode", options.delete_mode.as_str())
        );
        println!(
            "{}",
            ui.key_value("branches pending", &grouped.total.to_string())
        );

        if !grouped.both.is_empty() {
            println!(
                "{}",
                ui.key_value(
                    "local + remote",
                    &format!("{} branch(es)", grouped.both.len())
                )
            );
            println!("{}", format_preview_list(&grouped.both));
        }

        if !grouped.local_only.is_empty() {
            println!(
                "{}",
                ui.key_value(
                    "local only",
                    &format!("{} branch(es)", grouped.local_only.len())
                )
            );
            println!("{}", format_preview_list(&grouped.local_only));
        }

        if !grouped.remote_only.is_empty() {
            println!(
                "{}",
                ui.key_value(
                    "remote only",
                    &format!("{} branch(es)", grouped.remote_only.len())
                )
            );
            println!("{}", format_preview_list(&grouped.remote_only));
        }

        print!("{}", ui.prompt("Continue? (Y/n) "));
        stdout().flush()?;

        // Read the user's response on continuing
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        match input.to_lowercase().as_ref() {
            "y\n" | "y\r\n" | "yes\n" | "yes\r\n" | "\n" | "\r\n" => Ok(()),
            _ => Err(Error::ExitEarly),
        }
    }

    pub fn target_count(&self, options: &Options) -> Result<usize, Error> {
        Ok(self.grouped_preview(options)?.total)
    }

    pub fn merged(options: &Options) -> Result<Branches, Error> {
        let mut branches: Vec<String> = vec![];
        let ui = Ui::new(options.color_mode);
        println!(
            "{}",
            ui.info(&format!("Updating remote {}", options.remote))
        );
        run_command_with_no_output(&["git", "remote", "update", &options.remote, "--prune"])?;

        let escaped_base_branch = regex::escape(&options.base_branch);

        let merged_branches_regex = format!("^\\*?\\s*{}$", &escaped_base_branch);
        let merged_branches_filter =
            Regex::new(&merged_branches_regex).map_err(|source| Error::InvalidPattern {
                field: "base branch",
                value: options.base_branch.clone(),
                source,
            })?;
        let merged_branches_cmd = run_command(&["git", "branch", "--merged"])?;
        let merged_branches_output =
            String::from_utf8(merged_branches_cmd.stdout).map_err(|source| {
                Error::CommandOutputEncoding {
                    command: "git branch --merged".to_owned(),
                    source,
                }
            })?;

        let merged_branches =
            merged_branches_output
                .lines()
                .fold(Vec::<String>::new(), |mut acc, line| {
                    if !merged_branches_filter.is_match(line) {
                        acc.push(line.trim().to_string());
                    }
                    acc
                });

        let local_branches_regex = format!("^\\*?\\s*{}$", &escaped_base_branch);
        let local_branches_filter =
            Regex::new(&local_branches_regex).map_err(|source| Error::InvalidPattern {
                field: "base branch",
                value: options.base_branch.clone(),
                source,
            })?;
        let local_branches_cmd = run_command(&["git", "branch"])?;
        let local_branches_output =
            String::from_utf8(local_branches_cmd.stdout).map_err(|source| {
                Error::CommandOutputEncoding {
                    command: "git branch".to_owned(),
                    source,
                }
            })?;

        let local_branches = local_branches_output
            .lines()
            .fold(Vec::<String>::new(), |mut acc, line| {
                if !local_branches_filter.is_match(line) {
                    acc.push(line.trim().to_string());
                }
                acc
            })
            .iter()
            .filter(|branch| !options.ignored_branches.contains(branch))
            .cloned()
            .collect::<Vec<String>>();

        let remote_branches_regex = format!("\\b(HEAD|{})\\b", &escaped_base_branch);
        let remote_branches_filter =
            Regex::new(&remote_branches_regex).map_err(|source| Error::InvalidPattern {
                field: "base branch",
                value: options.base_branch.clone(),
                source,
            })?;
        let remote_branches_cmd = run_command(&["git", "branch", "-r"])?;
        let remote_branches_output =
            String::from_utf8(remote_branches_cmd.stdout).map_err(|source| {
                Error::CommandOutputEncoding {
                    command: "git branch -r".to_owned(),
                    source,
                }
            })?;

        let remote_branches =
            remote_branches_output
                .lines()
                .fold(Vec::<String>::new(), |mut acc, line| {
                    if !remote_branches_filter.is_match(line) {
                        acc.push(line.trim().to_string());
                    }
                    acc
                });

        let tracking = local_branch_tracking()?;

        for branch in local_branches {
            let tracking_info = tracking.get(&branch).cloned().unwrap_or_default();

            // Branch tracks a remote ref that no longer exists.
            if tracking_info.upstream_gone {
                branches.push(branch.to_owned());
                continue;
            }

            // First check if the local branch doesn't exist in the remote, it's the cheapest and easiest
            // way to determine if we want to suggest to delete it.
            if options.delete_unpushed_branches
                && !tracking_info.has_upstream
                && !remote_branches
                    .iter()
                    .any(|b: &String| *b == format!("{}/{}", &options.remote, branch))
            {
                branches.push(branch.to_owned());
                continue;
            }

            // If it does exist in the remote, check to see if it's listed in git branches --merged. If
            // it is, that means it wasn't merged using Github squashes, and we can suggest it.
            if merged_branches.contains(&branch) {
                branches.push(branch.to_owned());
                continue;
            }

            // If neither of the above matched, merge main into the branch and see if it succeeds.
            // If it can't cleanly merge, then it has likely been merged with Github squashes, and we
            // can suggest it.
            if options.squashes {
                run_command(&["git", "checkout", &branch])?;
                match run_command_with_status(&[
                    "git",
                    "pull",
                    "--ff-only",
                    &options.remote,
                    &options.base_branch,
                ]) {
                    Ok(status) => {
                        if !status.success() {
                            println!(
                                "{}",
                                ui.warning(&format!(
                                    "Branch {} appears to diverge from {}.",
                                    branch, options.base_branch
                                ))
                            );
                            branches.push(branch);
                        }
                    }
                    Err(err) => {
                        println!(
                            "{}",
                            ui.warning(&format!(
                                "Encountered error trying to update branch {} with branch {}: {}",
                                branch, options.base_branch, err
                            ))
                        );
                        continue;
                    }
                }

                run_command(&["git", "reset", "--hard"])?;
                run_command(&["git", "checkout", &options.base_branch])?;
            }
        }

        // if deleted in remote, list
        //
        // g branch -d -r <remote>/<branch>
        // g branch -d <branch>

        Ok(Branches::new(branches))
    }

    fn grouped_preview(&self, options: &Options) -> Result<GroupedPreview, Error> {
        let remote_branches = remote_branch_names(options)?;
        let mut grouped = GroupedPreview::default();

        match options.delete_mode {
            DeleteMode::Local => {
                grouped.local_only = self.vec.clone();
            }
            DeleteMode::Remote => {
                grouped.remote_only = self
                    .vec
                    .iter()
                    .filter(|branch| remote_branches.contains(*branch))
                    .cloned()
                    .collect();
            }
            DeleteMode::Both => {
                for branch in &self.vec {
                    if remote_branches.contains(branch) {
                        grouped.both.push(branch.clone());
                    } else {
                        grouped.local_only.push(branch.clone());
                    }
                }
            }
        }

        grouped.total = grouped.both.len() + grouped.local_only.len() + grouped.remote_only.len();
        Ok(grouped)
    }

    pub fn delete(&self, options: &Options) -> Result<String, Error> {
        if options.dry_run {
            return self.dry_run_message(options);
        }

        match options.delete_mode {
            DeleteMode::Local => delete_local_branches(self),
            DeleteMode::Remote => delete_remote_branches(self, options),
            DeleteMode::Both => {
                let local_output = delete_local_branches(self)?;
                let remote_output = delete_remote_branches(self, options)?;
                Ok([
                    "Remote:".to_owned(),
                    remote_output,
                    "\nLocal:".to_owned(),
                    local_output,
                ]
                .join("\n"))
            }
        }
    }

    fn dry_run_message(&self, options: &Options) -> Result<String, Error> {
        let grouped = self.grouped_preview(options)?;
        let mut rows = vec!["Dry run enabled: no branches were deleted.".to_owned()];

        if !grouped.both.is_empty() {
            rows.push(format!(
                "Would delete locally and remotely: {}",
                grouped.both.join(", ")
            ));
        }

        if !grouped.local_only.is_empty() {
            rows.push(format!(
                "Would delete locally: {}",
                grouped.local_only.join(", ")
            ));
        }

        if !grouped.remote_only.is_empty() {
            rows.push(format!(
                "Would delete remotely: {}",
                grouped.remote_only.join(", ")
            ));
        }

        Ok(rows.join("\n"))
    }
}

#[derive(Default)]
struct GroupedPreview {
    both: Vec<String>,
    local_only: Vec<String>,
    remote_only: Vec<String>,
    total: usize,
}

fn format_preview_list(branches: &[String]) -> String {
    let mut rows: Vec<String> = branches
        .iter()
        .take(PREVIEW_BRANCH_LIMIT)
        .map(|branch| format!("  - {}", branch))
        .collect();

    if branches.len() > PREVIEW_BRANCH_LIMIT {
        rows.push(format!(
            "  ... and {} more",
            branches.len() - PREVIEW_BRANCH_LIMIT
        ));
    }

    rows.join("\n")
}

fn remote_branch_names(options: &Options) -> Result<BTreeSet<String>, Error> {
    let cmd = run_command(&["git", "branch", "-r"])?;
    let output = String::from_utf8(cmd.stdout).map_err(|source| Error::CommandOutputEncoding {
        command: "git branch -r".to_owned(),
        source,
    })?;
    let remote_prefix = format!("{}/", options.remote);

    Ok(output
        .lines()
        .map(str::trim)
        .map(|line| line.trim_start_matches(&remote_prefix).to_owned())
        .collect())
}

#[derive(Clone, Copy, Default)]
struct BranchTracking {
    has_upstream: bool,
    upstream_gone: bool,
}

fn local_branch_tracking() -> Result<HashMap<String, BranchTracking>, Error> {
    let cmd = run_command(&[
        "git",
        "for-each-ref",
        "--format=%(refname:short)\t%(upstream:short)\t%(upstream:track)",
        "refs/heads",
    ])?;
    let output = String::from_utf8(cmd.stdout).map_err(|source| Error::CommandOutputEncoding {
        command: "git for-each-ref".to_owned(),
        source,
    })?;

    Ok(output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let branch = fields.next()?.trim();
            if branch.is_empty() {
                return None;
            }

            let upstream = fields.next().unwrap_or_default().trim();
            let track = fields.next().unwrap_or_default().trim();
            let has_upstream = !upstream.is_empty();
            let upstream_gone = has_upstream && track.contains("gone");

            Some((
                branch.to_owned(),
                BranchTracking {
                    has_upstream,
                    upstream_gone,
                },
            ))
        })
        .collect())
}

#[cfg(test)]
mod test {
    use super::{format_preview_list, Branches, PREVIEW_BRANCH_LIMIT};

    #[test]
    fn test_branches_new() {
        let input = vec!["branch1".to_owned(), "branch2".to_owned()];
        let branches = Branches::new(input);

        assert_eq!(
            vec!["branch1".to_owned(), "branch2".to_owned()],
            branches.vec
        );
    }

    #[test]
    fn test_format_preview_uses_bullets() {
        let input = vec!["branch1".to_owned(), "branch2".to_owned()];
        let branches = Branches::new(input);

        assert_eq!(
            "  - branch1\n  - branch2",
            format_preview_list(&branches.vec)
        );
    }

    #[test]
    fn test_format_preview_truncates_after_limit() {
        let input = (0..(PREVIEW_BRANCH_LIMIT + 3))
            .map(|i| format!("branch-{i}"))
            .collect::<Vec<String>>();
        let branches = Branches::new(input);
        let output = format_preview_list(&branches.vec);

        assert!(output.contains("  - branch-0"));
        assert!(output.contains(&format!("  - branch-{}", PREVIEW_BRANCH_LIMIT - 1)));
        assert!(output.contains("  ... and 3 more"));
        assert!(!output.contains(&format!("  - branch-{}", PREVIEW_BRANCH_LIMIT + 1)));
    }
}
