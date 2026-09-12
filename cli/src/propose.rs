use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::git;
use crate::okf;

pub struct Outcome {
    pub rel: String,
    pub committed: bool,
    pub pushed: bool,
    pub dry_run: bool,
}

#[derive(Serialize)]
pub struct ProposeJson {
    pub concept: String,
    pub agent: String,
    pub committed: bool,
    pub pushed: bool,
    pub dry_run: bool,
}

/// Create or overwrite one concept. Never deletes files.
pub fn run(
    root: &Path,
    concept: &str,
    agent: &str,
    markdown: &str,
    title: Option<&str>,
    push: bool,
    dry_run: bool,
) -> Result<Outcome> {
    if dry_run {
        let rel = okf::validate_propose(concept, markdown)?;
        return Ok(Outcome {
            rel,
            committed: false,
            pushed: false,
            dry_run: true,
        });
    }
    let written = okf::write_concept_markdown(root, concept, markdown)?;
    let path: PathBuf = root.join(&written.rel);
    let msg = title
        .map(|t| t.to_string())
        .unwrap_or_else(|| format!("okfsync: {} updated {}", agent, written.rel));
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
        dry_run: false,
    })
}

pub fn print_outcome(out: &Outcome, agent: &str, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(&ProposeJson {
                concept: out.rel.clone(),
                agent: agent.to_string(),
                committed: out.committed,
                pushed: out.pushed,
                dry_run: out.dry_run,
            })
            .context("json propose")?
        );
        return Ok(());
    }
    if out.dry_run {
        println!("dry-run '{}'", out.rel);
        println!("  agent:     {agent}");
        println!("  committed: false");
        println!("  pushed:    false");
        return Ok(());
    }
    if !out.committed {
        println!("unchanged '{}'", out.rel);
        println!("  agent:     {agent}");
        println!("  committed: false");
        println!("  pushed:    false");
    } else {
        println!("proposed '{}'", out.rel);
        println!("  agent:     {agent}");
        println!("  committed: true");
        println!("  pushed:    {}", out.pushed);
    }
    Ok(())
}
