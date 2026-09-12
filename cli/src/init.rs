use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::git;

const SAMPLE_BRAIN: &str = r#"---
type: Playbook
title: Shared Brain
description: Get, then propose. History lives in git on the server.
tags:
  - core
---

# Shared Brain

Seed concept. Agents `kbsync get` then `kbsync propose --file`. The CLI cannot delete.
"#;

const CONFIG: &str = r#"# okfsync knowledge root
default_branch = "main"
knowledge_root = "."
"#;

pub fn run(root: &Path, json: bool) -> Result<()> {
    fs::create_dir_all(root.join(".okfsync"))?;
    fs::create_dir_all(root.join("concepts"))?;

    let cfg = root.join(".okfsync/config.toml");
    if !cfg.exists() {
        fs::write(&cfg, CONFIG)?;
    }
    let tokens = root.join(".okfsync/tokens.toml");
    if !tokens.exists() {
        fs::write(&tokens, "# per-agent bearer tokens (hashes only)\n")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tokens, fs::Permissions::from_mode(0o600));
        }
    }

    let brain = root.join("concepts/brain.md");
    if !brain.exists() && okf_empty(root) {
        fs::write(&brain, SAMPLE_BRAIN)?;
    }

    if !git::is_git_repo(root) {
        let cfg = Config::load(root)?;
        git::run_git_ok(root, &["init", "-b", &cfg.default_branch])?;
        let _ = git::run_git_ok(root, &["config", "user.email", "okfsync@localhost"]);
        let _ = git::run_git_ok(root, &["config", "user.name", "okfsync"]);
        git::run_git_ok(root, &["add", "."])?;
        git::run_git_ok(root, &["commit", "-m", "okfsync init"])?;
    }

    if !root.join("concepts").is_dir() {
        bail!("init failed: concepts/ missing");
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "root": root.display().to_string(),
            })
        );
        return Ok(());
    }

    println!("initialized okfsync KB at {}", root.display());
    println!("next:");
    println!(
        "  kbsync token create --agent <id> --root {}",
        root.display()
    );
    println!("  kbsync serve --root {}", root.display());
    Ok(())
}

fn okf_empty(root: &Path) -> bool {
    match fs::read_dir(root.join("concepts")) {
        Ok(rd) => rd.filter_map(|e| e.ok()).count() == 0,
        Err(_) => true,
    }
}
