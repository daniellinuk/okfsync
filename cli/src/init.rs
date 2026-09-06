use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

use crate::git;

const SAMPLE_BRAIN: &str = r#"---
type: Playbook
title: Shared Brain
description: Claim before you write.
tags:
  - core
---

# Shared Brain

Seed concept. Agents `bagsy get` / `claim` / `propose` / `release` through the server.
"#;

const CONFIG: &str = r#"# bagsy knowledge root
default_branch = "main"
lock_dir = ".bagsy/locks"
knowledge_root = "."
"#;

pub fn run(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join(".bagsy/locks"))?;
    fs::create_dir_all(root.join("concepts"))?;

    let cfg = root.join(".bagsy/config.toml");
    if !cfg.exists() {
        fs::write(&cfg, CONFIG)?;
    }
    let gitkeep = root.join(".bagsy/locks/.gitkeep");
    if !gitkeep.exists() {
        fs::write(&gitkeep, "")?;
    }
    let tokens = root.join(".bagsy/tokens.toml");
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
        git::run_git_ok(root, &["init", "-b", "main"])?;
        let _ = git::run_git_ok(root, &["config", "user.email", "bagsy@localhost"]);
        let _ = git::run_git_ok(root, &["config", "user.name", "bagsy"]);
        git::run_git_ok(root, &["add", "."])?;
        git::run_git_ok(root, &["commit", "-m", "bagsy init"])?;
    }

    if !root.join("concepts").is_dir() {
        bail!("init failed: concepts/ missing");
    }

    println!("initialized bagsy KB at {}", root.display());
    println!("next:");
    println!("  bagsy token create --agent <id> --root {}", root.display());
    println!("  bagsy serve --root {}", root.display());
    Ok(())
}

fn okf_empty(root: &Path) -> bool {
    match fs::read_dir(root.join("concepts")) {
        Ok(rd) => rd.filter_map(|e| e.ok()).count() == 0,
        Err(_) => true,
    }
}
