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

pub fn commit_paths(root: &Path, paths: &[&Path], message: &str, agent: &str) -> Result<bool> {
    if !is_git_repo(root) {
        return Ok(false);
    }
    for p in paths {
        let rel = p.strip_prefix(root).unwrap_or(p);
        let rel_s = rel.to_string_lossy().into_owned();
        run_git_ok(root, &["add", "--", &rel_s])?;
    }
    let status = status_porcelain(root)?;
    if status.is_empty() {
        return Ok(false);
    }
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", agent)
        .env("GIT_AUTHOR_EMAIL", format!("{agent}@bagsy.local"))
        .env("GIT_COMMITTER_NAME", agent)
        .env("GIT_COMMITTER_EMAIL", format!("{agent}@bagsy.local"))
        .output()
        .context("git commit")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git commit failed: {}", stderr.trim());
    }
    Ok(true)
}

pub fn push_head(root: &Path) -> Result<()> {
    let branch = current_branch(root)?;
    push_branch(root, &branch, false)
}
