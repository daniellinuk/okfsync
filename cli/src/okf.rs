use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    /// Required by OKF v0.1 — the only mandatory field.
    pub r#type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Concept {
    pub rel: String,
    pub frontmatter: Frontmatter,
    pub body: String,
}

pub fn normalize_concept_path(concept: &str) -> String {
    let mut c = concept.trim().trim_start_matches("./").to_string();
    if c.starts_with('/') {
        c = c.trim_start_matches('/').to_string();
    }
    if !c.ends_with(".md") {
        c.push_str(".md");
    }
    // Prefer concepts/ prefix when a bare name is given
    if !c.contains('/') && !c.starts_with("concepts/") {
        c = format!("concepts/{c}");
    }
    c
}

pub fn parse_frontmatter(text: &str) -> Result<(Frontmatter, String)> {
    let text = text.trim_start_matches('\u{feff}');
    if !text.starts_with("---") {
        bail!("missing YAML frontmatter (OKF requires --- type: ... ---)");
    }
    let rest = &text[3..];
    let end = rest
        .find("\n---")
        .context("unterminated YAML frontmatter")?;
    let yaml = rest[..end].trim();
    let body = rest[end + 4..].trim_start_matches('\n').to_string();
    let fm: Frontmatter = serde_yaml::from_str(yaml).context("parsing OKF frontmatter")?;
    if fm.r#type.trim().is_empty() {
        bail!("frontmatter `type` must be a non-empty string");
    }
    Ok((fm, body))
}

pub fn read_concept(root: &Path, concept: &str) -> Result<Concept> {
    let rel = normalize_concept_path(concept);
    let path = root.join(&rel);
    if !path.exists() {
        bail!("concept not found: {} ({})", rel, path.display());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let (frontmatter, body) = parse_frontmatter(&text)
        .with_context(|| format!("invalid OKF concept {}", rel))?;
    Ok(Concept {
        rel,
        frontmatter,
        body,
    })
}

pub fn list_concepts(root: &Path) -> Result<Vec<PathBuf>> {
    let concepts_dir = root.join("concepts");
    if !concepts_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(&concepts_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            // skip index.md / log.md conventionals from "concept lint as editable units" optionally
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "index.md" || name == "log.md" {
                continue;
            }
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

pub fn rel_from_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}
