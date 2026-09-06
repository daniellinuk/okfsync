use anyhow::Result;
use std::path::Path;

use crate::okf;

pub fn run(root: &Path, concept: &str) -> Result<()> {
    let c = okf::read_concept(root, concept)?;
    println!("# {}", c.rel);
    println!("type: {}", c.frontmatter.r#type);
    if let Some(title) = &c.frontmatter.title {
        println!("title: {title}");
    }
    if let Some(desc) = &c.frontmatter.description {
        println!("description: {desc}");
    }
    if !c.frontmatter.tags.is_empty() {
        println!("tags: {}", c.frontmatter.tags.join(", "));
    }
    println!();
    print!("{}", c.body);
    if !c.body.ends_with('\n') {
        println!();
    }
    Ok(())
}
