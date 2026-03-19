use crate::support::project;

#[test]
fn test_git_clean_removes_local_branches() {
    let project = project("git-clean_removes_local").build();

    project.setup_command("git branch test1");
    project.setup_command("git branch test2");

    let verify = project.setup_command("git branch");

    assert!(
        verify.stdout().contains("test1"),
        "{}",
        verify.failure_message("test1")
    );
    assert!(
        verify.stdout().contains("test2"),
        "{}",
        verify.failure_message("test2")
    );

    let result = project.git_clean_command("-y").run();

    assert!(
        result.is_success(),
        "{}",
        result.failure_message("command to succeed")
    );
    assert!(
        result.stdout().contains("Deleted branch test1"),
        "{}",
        result.failure_message("command to delete test1")
    );
    assert!(
        result.stdout().contains("Deleted branch test2"),
        "{}",
        result.failure_message("command to delete test2")
    );
}

#[test]
fn test_git_clean_does_not_remove_ignored_local_branches() {
    let project = project("git-clean_removes_local").build();

    project.setup_command("git branch test1");
    project.setup_command("git branch test2");

    let verify = project.setup_command("git branch");

    assert!(
        verify.stdout().contains("test1"),
        "{}",
        verify.failure_message("test1")
    );
    assert!(
        verify.stdout().contains("test2"),
        "{}",
        verify.failure_message("test2")
    );

    let result = project.git_clean_command("-y -i test2").run();

    assert!(
        result.is_success(),
        "{}",
        result.failure_message("command to succeed")
    );
    assert!(
        result.stdout().contains("Deleted branch test1"),
        "{}",
        result.failure_message("command to delete test1")
    );
    assert!(
        !result.stdout().contains("Deleted branch test2"),
        "{}",
        result.failure_message("command to delete test2")
    );
}

#[test]
fn test_git_clean_does_not_remove_list_of_ignored_local_branches() {
    let project = project("git-clean_removes_local").build();

    project.setup_command("git branch test1");
    project.setup_command("git branch test2");
    project.setup_command("git branch test3");

    let verify = project.setup_command("git branch");

    assert!(
        verify.stdout().contains("test1"),
        "{}",
        verify.failure_message("test1")
    );
    assert!(
        verify.stdout().contains("test2"),
        "{}",
        verify.failure_message("test2")
    );
    assert!(
        verify.stdout().contains("test3"),
        "{}",
        verify.failure_message("test3")
    );

    let result = project.git_clean_command("-y -i test1 -i test3").run();

    assert!(
        result.is_success(),
        "{}",
        result.failure_message("command to succeed")
    );
    assert!(
        !result.stdout().contains("Deleted branch test1"),
        "{}",
        result.failure_message("command to delete test1")
    );
    assert!(
        result.stdout().contains("Deleted branch test2"),
        "{}",
        result.failure_message("command to delete test2")
    );
    assert!(
        !result.stdout().contains("Deleted branch test3"),
        "{}",
        result.failure_message("command to delete test3")
    );
}

#[test]
fn test_git_clean_handles_base_branch_regex_metacharacters() {
    let project = project("git-clean_base_branch_regex_metacharacters").build();

    project.setup_command("git branch -m main(test)");

    let result = project.git_clean_command("-y -b main(test)").run();

    assert!(
        result.is_success(),
        "{}",
        result.failure_message("command to succeed")
    );
    assert!(
        result
            .stdout()
            .contains("No branches to delete, you're clean!"),
        "{}",
        result.failure_message("no branches to delete")
    );
}
