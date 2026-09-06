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

pub fn normalize_concept_path(concept: &str) -> Result<String> {
    let mut c = concept.trim().trim_start_matches("./").to_string();
    if c.starts_with('/') {
        c = c.trim_start_matches('/').to_string();
    }
    c = c.replace('\\', "/");
    if !c.ends_with(".md") {
        c.push_str(".md");
    }
    if !c.contains('/') && !c.starts_with("concepts/") {
        c = format!("concepts/{c}");
    }
    assert_safe_concept_rel(&c)?;
    Ok(c)
}

/// Concepts may only be created/updated under `concepts/**/*.md`. No deletes, no `..`.
pub fn assert_safe_concept_rel(rel: &str) -> Result<()> {
    let rel = rel.replace('\\', "/");
    if rel.is_empty() || rel.starts_with('/') || rel.contains('\0') {
        bail!("invalid concept path");
    }
    if !rel.starts_with("concepts/") || !rel.ends_with(".md") {
        bail!("concepts must be markdown files under concepts/");
    }
    for part in rel.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            bail!("refusing concept path '{rel}'");
        }
    }
    Ok(())
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
    let rel = normalize_concept_path(concept)?;
    let path = root.join(&rel);
    ensure_under_concepts(root, &path)?;
    if !path.exists() {
        bail!("concept not found: {} ({})", rel, path.display());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let (frontmatter, body) =
        parse_frontmatter(&text).with_context(|| format!("invalid OKF concept {}", rel))?;
    Ok(Concept {
        rel,
        frontmatter,
        body,
    })
}

/// Write a full markdown document after validating OKF frontmatter.
/// Never deletes; only creates or overwrites a `.md` file under `concepts/`.
pub fn write_concept_markdown(root: &Path, concept: &str, markdown: &str) -> Result<Concept> {
    let rel = normalize_concept_path(concept)?;
    let (frontmatter, body) =
        parse_frontmatter(markdown).with_context(|| format!("invalid OKF concept {rel}"))?;
    let path = root.join(&rel);
    ensure_under_concepts(root, &path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut text = markdown.to_string();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(Concept {
        rel,
        frontmatter,
        body,
    })
}

fn ensure_under_concepts(root: &Path, path: &Path) -> Result<()> {
    let concepts = root.join("concepts");
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let concepts_abs = concepts
        .canonicalize()
        .unwrap_or_else(|_| root_abs.join("concepts"));
    let candidate = if path.exists() {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    } else {
        let parent = path.parent().unwrap_or(path);
        let parent_abs = parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf());
        parent_abs.join(path.file_name().unwrap_or_default())
    };
    if !candidate.starts_with(&concepts_abs) {
        bail!(
            "refusing to touch path outside concepts/: {}",
            path.display()
        );
    }
    Ok(())
}

pub fn list_concepts(root: &Path) -> Result<Vec<PathBuf>> {
    let concepts_dir = root.join("concepts");
    if !concepts_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(&concepts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
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
