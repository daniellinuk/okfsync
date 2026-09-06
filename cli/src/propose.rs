use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::git;
use crate::lock;
use crate::okf;

pub struct Outcome {
    pub rel: String,
    pub committed: bool,
    pub pushed: bool,
}

pub fn run(
    root: &Path,
    concept: &str,
    agent: &str,
    markdown: &str,
    title: Option<&str>,
    push: bool,
) -> Result<Outcome> {
    let cfg = Config::load(root)?;
    let rel = lock::normalize_concept(concept);
    let lock_file = lock::lock_path(root, &cfg, &rel);
    match lock::read_lock(&lock_file)? {
        Some(existing) if existing.agent == agent => {}
        Some(existing) => bail!(
            "concept '{rel}' is bagsied by '{}' (you are '{agent}')",
            existing.agent
        ),
        None => bail!("concept '{rel}' is not claimed — run bagsy claim first"),
    }
    let written = okf::write_concept_markdown(root, &rel, markdown)?;
    let path: PathBuf = root.join(&written.rel);
    let msg = title
        .map(|t| t.to_string())
        .unwrap_or_else(|| format!("bagsy: {} updated {}", agent, written.rel));
    let committed = git::commit_paths(root, &[path.as_path()], &msg, agent)?;
    let mut pushed = false;
    if push && committed && git::has_remote(root, "origin") {
        git::push_head(root)?;
        pushed = true;
    }
    Ok(Outcome {
        rel: written.rel,
        committed,
        pushed,
    })
}

pub fn print_outcome(out: &Outcome, agent: &str) {
    if !out.committed {
        println!(
            "wrote '{}' (no git commit — nothing changed or not a repo)",
            out.rel
        );
    } else {
        println!("proposed '{}'", out.rel);
        println!("  agent:     {agent}");
        println!("  committed: {}", out.committed);
        println!("  pushed:    {}", out.pushed);
    }
}
