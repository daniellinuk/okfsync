use anyhow::{Context, Result};
use std::path::Path;

use crate::api::{ConceptSummary, PagesResponse};
use crate::okf;

pub fn print_pages(concepts: &[ConceptSummary], json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(&PagesResponse {
                concepts: concepts.to_vec(),
            })
            .context("json pages")?
        );
        return Ok(());
    }
    if concepts.is_empty() {
        println!("no concepts");
        println!("  kbsync propose brain --file ./brain.md");
        return Ok(());
    }
    for c in concepts {
        let title = c.title.as_deref().unwrap_or("-");
        let kind = if c.r#type.is_empty() { "-" } else { &c.r#type };
        println!("{}\t{}\t{}", c.rel, kind, title);
    }
    Ok(())
}

pub fn print_search_hits(concepts: &[ConceptSummary], json: bool) -> Result<()> {
    if json {
        return print_pages(concepts, true);
    }
    if concepts.is_empty() {
        println!("no matches");
        println!("  kbsync list");
        println!("  kbsync search <query>");
        return Ok(());
    }
    print_pages(concepts, false)
}

pub fn run_list(root: &Path, json: bool) -> Result<()> {
    print_pages(&okf::summaries(root)?, json)
}

pub fn run_search(root: &Path, query: &str, json: bool) -> Result<()> {
    print_search_hits(&okf::search_concepts(root, query)?, json)
}
