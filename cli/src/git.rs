use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Output};

use crate::okf;

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

pub fn push_branch(root: &Path, branch: &str) -> Result<()> {
    run_git_ok(root, &["push", "-u", "origin", branch])?;
    Ok(())
}

pub fn has_remote(root: &Path, name: &str) -> bool {
    run_git(root, &["remote", "get-url", name])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Stage and commit only existing concept files. Never `git rm`, `-A`, or force-push.
pub fn commit_paths(root: &Path, paths: &[&Path], message: &str, agent: &str) -> Result<bool> {
    if !is_git_repo(root) {
        return Ok(false);
    }
    let mut rels: Vec<String> = Vec::new();
    for p in paths {
        if !p.is_file() {
            bail!(
                "refusing to stage missing path (kbsync never deletes): {}",
                p.display()
            );
        }
        let rel = p
            .strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        okf::assert_safe_concept_rel(&rel)?;
        rels.push(rel);
    }
    for rel in &rels {
        run_git_ok(root, &["add", "--", rel])?;
    }
    let mut diff_args: Vec<&str> = vec!["diff", "--cached", "--quiet", "--"];
    for rel in &rels {
        diff_args.push(rel.as_str());
    }
    let diff = run_git(root, &diff_args)?;
    match diff.status.code() {
        Some(0) => return Ok(false),
        Some(1) => {}
        _ => {
            let stderr = String::from_utf8_lossy(&diff.stderr);
            bail!("git diff --cached failed: {}", stderr.trim());
        }
    }

    let mut cmd = Command::new("git");
    cmd.arg("commit").arg("-m").arg(message).arg("--");
    for rel in &rels {
        cmd.arg(rel);
    }
    let output = cmd
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", agent)
        .env("GIT_AUTHOR_EMAIL", format!("{agent}@okfsync.local"))
        .env("GIT_COMMITTER_NAME", agent)
        .env("GIT_COMMITTER_EMAIL", format!("{agent}@okfsync.local"))
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
    push_branch(root, &branch)
}

/// Last git author + author date for a concept file. Best-effort; None if no repo/history.
pub fn file_provenance(root: &Path, rel: &str) -> (Option<String>, Option<String>) {
    if !is_git_repo(root) {
        return (None, None);
    }
    let output = match run_git(root, &["log", "-1", "--format=%an%x09%aI", "--", rel]) {
        Ok(o) if o.status.success() => o,
        _ => return (None, None),
    };
    let line = String::from_utf8_lossy(&output.stdout);
    let line = line.trim();
    if line.is_empty() {
        return (None, None);
    }
    match line.split_once('\t') {
        Some((name, at)) if !name.is_empty() => (Some(name.to_string()), Some(at.to_string())),
        _ => (None, None),
    }
}
