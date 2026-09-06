use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::git;

pub fn run(root: &Path, title: Option<&str>, force_with_lease: bool) -> Result<()> {
    let cfg = Config::load(root)?;
    let git_root = git::find_git_toplevel(root).unwrap_or_else(|| root.to_path_buf());

    if !git::is_git_repo(&git_root) {
        bail!("not a git repository: {}", git_root.display());
    }

    let branch = git::current_branch(&git_root)?;
    if cfg.is_protected_branch(&branch) {
        bail!(
            "refusing to propose from protected branch '{branch}'.\n\
bagsy workers never push main — run `bagsy claim <concept>` first."
        );
    }

    if !branch.starts_with("bagsy/") {
        eprintln!(
            "warning: branch '{branch}' does not look like a bagsy/* claim branch"
        );
    }

    let status = git::status_porcelain(&git_root)?;
    if !status.is_empty() {
        eprintln!("working tree has uncommitted changes:");
        eprintln!("{status}");
        eprintln!("commit your concept edits before proposing (bagsy does not auto-commit).");
        bail!("uncommitted changes — commit first, then re-run bagsy propose");
    }

    let title = title.unwrap_or("bagsy: propose concept update");

    if git::has_remote(&git_root, "origin") {
        match git::push_branch(&git_root, &branch, force_with_lease) {
            Ok(()) => println!("pushed branch '{branch}' to origin"),
            Err(e) => {
                eprintln!("push failed: {e}");
                eprintln!("continue with a manual push, then open a PR/MR.");
            }
        }
    } else {
        println!("no 'origin' remote — skipped push");
    }

    println!();
    println!("Propose a PR/MR (never merge by pushing main):");
    println!("  title: {title}");
    println!("  head:  {branch}");
    println!("  base:  {}", cfg.default_branch);
    println!();

    // Prefer origin CLI (private Origin) then gh
    if command_exists("origin") {
        println!("hint: origin pr create --title \"{title}\" --head {branch} --base {}", cfg.default_branch);
    } else if command_exists("gh") {
        println!("hint: gh pr create --title \"{title}\" --head {branch} --base {}", cfg.default_branch);
    } else {
        println!("hint: open a merge request from '{branch}' → '{}'", cfg.default_branch);
    }

    Ok(())
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
