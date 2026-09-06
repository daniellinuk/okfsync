use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;
use crate::lock;

pub fn run(root: &Path, concept: &str, agent: &str, force: bool) -> Result<()> {
    let cfg = Config::load(root)?;
    let rel = lock::normalize_concept(concept);
    let path = lock::lock_path(root, &cfg, &rel);

    match lock::read_lock(&path)? {
        None => bail!("no lock found for '{rel}'"),
        Some(existing) => {
            if existing.agent != agent && !force {
                bail!(
                    "lock for '{rel}' is held by '{}' (you are '{agent}'). Use --force to override.",
                    existing.agent
                );
            }
            lock::remove_lock(&path)?;
            println!(
                "released '{rel}' (was held by {})",
                existing.agent
            );
        }
    }
    Ok(())
}
