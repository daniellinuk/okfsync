use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config::{self, Config};
use crate::git;
use crate::lock::{self, Lock};
use crate::okf;

pub fn run(root: &Path, concept: &str, agent: Option<&str>, no_branch: bool) -> Result<()> {
    let cfg = Config::load(root)?;
    let agent = config::default_agent(agent);
    let rel = lock::normalize_concept(concept);

    // Concept must exist
    let _ = okf::read_concept(root, &rel)?;

    lock::ensure_claimable(root, &cfg, &rel, &agent)?;

    let branch = lock::branch_name(&agent, &rel);
    let lock_file = lock::lock_path(root, &cfg, &rel);
    let claim = Lock::new(&rel, &agent, &branch);
    lock::write_lock(&lock_file, &claim)?;

    if !no_branch {
        let git_root = git::find_git_toplevel(root).unwrap_or_else(|| root.to_path_buf());
        if !git::is_git_repo(&git_root) {
            bail!(
                "claimed lock written, but {} is not a git repo — pass --no-branch or init git",
                git_root.display()
            );
        }
        // Stage the lock so the claim is visible on the branch
        git::create_and_checkout_branch(&git_root, &branch)
            .with_context(|| format!("creating branch {branch}"))?;
        let _ = Command::new("git")
            .args(["add", "--"])
            .arg(&lock_file)
            .current_dir(&git_root)
            .status();
    }

    println!("bagsied '{rel}'");
    println!("  agent:  {agent}");
    println!("  branch: {branch}");
    println!("  lock:   {}", lock_file.strip_prefix(root).unwrap_or(&lock_file).display());
    println!();
    println!("Edit the concept, then: bagsy propose");
    Ok(())
}
