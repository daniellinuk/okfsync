use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Output};

pub fn run_git(root: &Path, args: &[&str]) -> Result<Output> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    Ok(output)
}

pub fn run_git_ok(root: &Path, args: &[&str]) -> Result<String> {
    let output = run_git(root, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn is_git_repo(root: &Path) -> bool {
    run_git(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn current_branch(root: &Path) -> Result<String> {
    run_git_ok(root, &["rev-parse", "--abbrev-ref", "HEAD"])
}

pub fn create_and_checkout_branch(root: &Path, branch: &str) -> Result<()> {
    // If branch exists locally, check it out; else create from HEAD
    let exists = run_git(root, &["rev-parse", "--verify", branch])?
        .status
        .success();
    if exists {
        run_git_ok(root, &["checkout", branch])?;
    } else {
        run_git_ok(root, &["checkout", "-b", branch])?;
    }
    Ok(())
}

pub fn push_branch(root: &Path, branch: &str, force_with_lease: bool) -> Result<()> {
    if force_with_lease {
        run_git_ok(root, &["push", "--force-with-lease", "-u", "origin", branch])?;
    } else {
        run_git_ok(root, &["push", "-u", "origin", branch])?;
    }
    Ok(())
}

pub fn has_remote(root: &Path, name: &str) -> bool {
    run_git(root, &["remote", "get-url", name])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn status_porcelain(root: &Path) -> Result<String> {
    run_git_ok(root, &["status", "--porcelain"])
}

pub fn find_git_toplevel(start: &Path) -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(s))
    }
}
