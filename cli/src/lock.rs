use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::okf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lock {
    pub concept: String,
    pub agent: String,
    pub branch: String,
    pub claimed_at: DateTime<Utc>,
}

impl Lock {
    pub fn new(concept: &str, agent: &str, branch: &str) -> Self {
        Self {
            concept: concept.to_string(),
            agent: agent.to_string(),
            branch: branch.to_string(),
            claimed_at: Utc::now(),
        }
    }
}

pub fn normalize_concept(concept: &str) -> String {
    okf::normalize_concept_path(concept)
}

pub fn lock_path(root: &Path, cfg: &Config, concept: &str) -> PathBuf {
    let key = normalize_concept(concept)
        .trim_end_matches(".md")
        .replace('/', "__");
    cfg.lock_dir_path(root).join(format!("{key}.lock"))
}

pub fn read_lock(path: &Path) -> Result<Option<Lock>> {
    if !path.exists() {
        return Ok(None);
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("reading lock {}", path.display()))?;
    let lock: Lock =
        toml::from_str(&text).with_context(|| format!("parsing lock {}", path.display()))?;
    Ok(Some(lock))
}

pub fn write_lock(path: &Path, lock: &Lock) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating lock dir {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(lock).context("serializing lock")?;
    fs::write(path, text).with_context(|| format!("writing lock {}", path.display()))?;
    Ok(())
}

pub fn remove_lock(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("removing lock {}", path.display()))?;
    }
    Ok(())
}

pub fn list_locks(root: &Path, cfg: &Config) -> Result<Vec<(PathBuf, Lock)>> {
    let dir = cfg.lock_dir_path(root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lock") {
            continue;
        }
        if let Some(lock) = read_lock(&path)? {
            out.push((path, lock));
        }
    }
    out.sort_by(|a, b| a.1.concept.cmp(&b.1.concept));
    Ok(out)
}

pub fn ensure_claimable(root: &Path, cfg: &Config, concept: &str, agent: &str) -> Result<()> {
    let path = lock_path(root, cfg, concept);
    if let Some(existing) = read_lock(&path)? {
        if existing.agent != agent {
            bail!(
                "concept '{}' is already bagsied by agent '{}' on branch '{}' (claimed {})",
                existing.concept,
                existing.agent,
                existing.branch,
                existing.claimed_at.to_rfc3339()
            );
        }
    }
    Ok(())
}

pub fn branch_name(agent: &str, concept: &str) -> String {
    let slug = normalize_concept(concept)
        .trim_end_matches(".md")
        .replace('/', "-");
    let agent_slug = agent
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>();
    format!("bagsy/{agent_slug}/{slug}")
}
