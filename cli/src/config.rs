use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_BRANCH: &str = "main";
pub const CONFIG_FILE: &str = ".okfsync/config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Relative knowledge root inside the repo (usually ".").
    #[serde(default = "default_knowledge_root")]
    #[allow(dead_code)]
    pub knowledge_root: String,
    #[serde(default = "default_branch")]
    pub default_branch: String,
}

fn default_knowledge_root() -> String {
    ".".into()
}
fn default_branch() -> String {
    DEFAULT_BRANCH.into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            knowledge_root: default_knowledge_root(),
            default_branch: default_branch(),
        }
    }
}

impl Config {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading config {}", path.display()))?;
        let cfg: Config =
            toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))?;
        Ok(cfg)
    }
}

/// Resolve the okfsync/OKF root: explicit flag, walk up for `.okfsync/`, or cwd.
pub fn resolve_root(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        let abs = if p.is_absolute() {
            p.to_path_buf()
        } else {
            std::env::current_dir()?.join(p)
        };
        if !abs.is_dir() {
            bail!("okfsync root is not a directory: {}", abs.display());
        }
        return Ok(abs);
    }

    let cwd = std::env::current_dir()?;
    let mut cur = cwd.as_path();
    loop {
        if cur.join(".okfsync").is_dir() || cur.join("concepts").is_dir() {
            return Ok(cur.to_path_buf());
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => break,
        }
    }
    Ok(cwd)
}

pub fn default_agent(explicit: Option<&str>) -> String {
    if let Some(a) = explicit {
        return a.to_string();
    }
    if let Ok(a) = std::env::var("KBSYNC_AGENT") {
        if !a.is_empty() {
            return a;
        }
    }
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "agent".into())
}
