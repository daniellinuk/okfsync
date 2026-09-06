use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

use crate::api::ConceptResponse;
use crate::okf;

#[derive(Debug, Clone, Serialize)]
pub struct GetJson {
    pub rel: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub body: String,
    pub markdown: String,
}

impl GetJson {
    pub fn from_document(doc: &okf::Document) -> Self {
        let c = &doc.concept;
        Self {
            rel: c.rel.clone(),
            r#type: c.frontmatter.r#type.clone(),
            title: c.frontmatter.title.clone(),
            description: c.frontmatter.description.clone(),
            tags: c.frontmatter.tags.clone(),
            body: c.body.clone(),
            markdown: doc.markdown.clone(),
        }
    }

    pub fn from_response(c: &ConceptResponse) -> Self {
        let markdown = if c.markdown.is_empty() {
            reconstruct_markdown(c)
        } else {
            c.markdown.clone()
        };
        Self {
            rel: c.rel.clone(),
            r#type: c.r#type.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            tags: c.tags.clone(),
            body: c.body.clone(),
            markdown,
        }
    }
}

fn reconstruct_markdown(c: &ConceptResponse) -> String {
    let mut s = String::from("---\n");
    s.push_str(&format!("type: {}\n", c.r#type));
    if let Some(title) = &c.title {
        s.push_str(&format!("title: {title}\n"));
    }
    if let Some(desc) = &c.description {
        s.push_str(&format!("description: {desc}\n"));
    }
    if !c.tags.is_empty() {
        s.push_str("tags:\n");
        for t in &c.tags {
            s.push_str(&format!("  - {t}\n"));
        }
    }
    s.push_str("---\n");
    if !c.body.is_empty() {
        s.push('\n');
        s.push_str(&c.body);
        if !c.body.ends_with('\n') {
            s.push('\n');
        }
    }
    s
}

pub fn print_markdown(markdown: &str) {
    print!("{markdown}");
    if !markdown.ends_with('\n') {
        println!();
    }
}

pub fn print_json(v: &GetJson) -> Result<()> {
    println!("{}", serde_json::to_string(v).context("json get")?);
    Ok(())
}

pub fn run(root: &Path, concept: &str, json: bool) -> Result<()> {
    let doc = okf::load_document(root, concept)?;
    if json {
        print_json(&GetJson::from_document(&doc))
    } else {
        print_markdown(&doc.markdown);
        Ok(())
    }
}

pub fn print_response(c: &ConceptResponse, json: bool) -> Result<()> {
    let out = GetJson::from_response(c);
    if json {
        print_json(&out)
    } else {
        print_markdown(&out.markdown);
        Ok(())
    }
}
