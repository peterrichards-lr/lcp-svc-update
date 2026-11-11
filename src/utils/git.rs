use std::path::Path;
use std::process::Command;

pub fn is_repository(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Returns `git status --porcelain` lines for any of `files` with uncommitted changes.
pub fn uncommitted_changes(root: &Path, files: &[&Path]) -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--"])
        .args(files)
        .current_dir(root)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Git status failed")
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect())
}

pub fn git_add(root: &Path, files: &[&Path]) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("add")
        .arg("--")
        .args(files)
        .current_dir(root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git add failed")
    }
}

/// Commits only `files`, leaving anything else that is staged untouched.
pub fn git_commit(root: &Path, message: &str, files: &[&Path]) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["commit", "-m", message, "--only", "--"])
        .args(files)
        .current_dir(root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git commit failed")
    }
}

pub fn git_push(root: &Path) -> anyhow::Result<()> {
    let status = Command::new("git").arg("push").current_dir(root).status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git push failed")
    }
}
