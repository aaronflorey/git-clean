use crate::support::project;

#[test]
fn test_git_clean_checks_for_git_in_path() {
    let project = project("git-clean_removes").build();

    let result = project.git_clean_command("-y").env("PATH", "").run();

    assert!(
        !result.is_success(),
        "{}",
        result.failure_message("command to fail")
    );
    assert!(
        result
            .stdout()
            .contains("Unable to execute 'git' on your machine"),
        "{}",
        result.failure_message("to be missing the git command")
    );
}

#[test]
fn test_git_clean_fails_when_remote_update_fails() {
    let project = project("git-clean_remote_update_failure").build();

    let result = project.git_clean_command("-y").run();

    assert!(
        !result.is_success(),
        "{}",
        result.failure_message("command to fail when remote update fails")
    );
    assert!(
        result
            .stdout()
            .contains("Command `git remote update origin --prune` failed"),
        "{}",
        result.failure_message("to include remote update failure context")
    );
}
