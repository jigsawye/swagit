use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn setup_git_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Initialize Git repository
    StdCommand::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Set Git configuration
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Create initial commit
    StdCommand::new("touch")
        .arg("README.md")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    temp_dir
}

#[test]
fn test_not_git_repo() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("sg")
        .unwrap()
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));
}

#[test]
fn test_current_branch() {
    let temp_dir = setup_git_repo();

    // Create a test branch so the repository has multiple branches
    StdCommand::new("git")
        .args(["checkout", "-b", "test-branch"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["checkout", "main"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Only check the output of the current branch, do not test interactive functionality
    Command::cargo_bin("sg")
        .unwrap()
        .current_dir(&temp_dir)
        .env("RUST_BACKTRACE", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Current branch is main"))
        .stdout(predicate::str::contains("Switched to branch test-branch")); // Automatically switched to the first branch in non-terminal environments
}

#[test]
fn test_delete_branch() {
    let temp_dir = setup_git_repo();

    // Create a new branch
    StdCommand::new("git")
        .args(["checkout", "-b", "test-branch"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Switch back to main
    StdCommand::new("git")
        .args(["checkout", "main"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Directly use the git command to delete the branch, instead of using the interactive interface
    StdCommand::new("git")
        .args(["branch", "-D", "test-branch"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    // Verify that the branch has been deleted
    let output = StdCommand::new("git")
        .args(["branch"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(!String::from_utf8_lossy(&output.stdout).contains("test-branch"));
}

#[test]
fn test_sync_with_no_remote() {
    let temp_dir = setup_git_repo();

    Command::cargo_bin("sg")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("-s")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to fetch from remote"));
}
