use anyhow::Result;
use std::path::Path;

use crate::okf;

pub fn render(c: &okf::Concept) -> String {
    let mut s = String::new();
    s.push_str(&format!("# {}\n", c.rel));
    s.push_str(&format!("type: {}\n", c.frontmatter.r#type));
    if let Some(title) = &c.frontmatter.title {
        s.push_str(&format!("title: {title}\n"));
    }
    if let Some(desc) = &c.frontmatter.description {
        s.push_str(&format!("description: {desc}\n"));
    }
    if !c.frontmatter.tags.is_empty() {
        s.push_str(&format!("tags: {}\n", c.frontmatter.tags.join(", ")));
    }
    s.push('\n');
    s.push_str(&c.body);
    if !c.body.ends_with('\n') {
        s.push('\n');
    }
    s
}

pub fn run(root: &Path, concept: &str) -> Result<()> {
    let c = okf::read_concept(root, concept)?;
    print!("{}", render(&c));
    Ok(())
}

pub fn print_response(c: &crate::api::ConceptResponse) {
    println!("# {}", c.rel);
    println!("type: {}", c.r#type);
    if let Some(title) = &c.title {
        println!("title: {title}");
    }
    if let Some(desc) = &c.description {
        println!("description: {desc}");
    }
    if !c.tags.is_empty() {
        println!("tags: {}", c.tags.join(", "));
    }
    println!();
    print!("{}", c.body);
    if !c.body.ends_with('\n') {
        println!();
    }
}
