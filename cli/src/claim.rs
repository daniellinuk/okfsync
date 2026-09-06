use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::lock;
use crate::okf;

pub fn run(root: &Path, concept: &str, agent: &str) -> Result<()> {
    let cfg = Config::load(root)?;
    let rel = lock::normalize_concept(concept);
    let _ = okf::read_concept(root, &rel)?;
    let claim = lock::try_claim(root, &cfg, &rel, agent)?;
    println!("bagsied '{}'", claim.concept);
    println!("  agent: {}", claim.agent);
    println!();
    println!("Edit, then: bagsy propose {} --file <markdown>", concept);
    Ok(())
}
