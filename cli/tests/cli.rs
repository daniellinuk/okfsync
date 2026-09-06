use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn write_concept(root: &std::path::Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn init_okf(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    fs::create_dir_all(root.join(".bagsy/locks")).unwrap();
    fs::write(
        root.join(".bagsy/config.toml"),
        "default_branch = \"main\"\nlock_dir = \".bagsy/locks\"\n",
    )
    .unwrap();
    write_concept(
        &root,
        "concepts/brain.md",
        r#"---
type: Playbook
title: Shared Brain
description: The swarm memory nucleus.
tags: [core]
---

# Shared Brain

Agents read and write here carefully.
"#,
    );
    write_concept(
        &root,
        "concepts/routing.md",
        r#"---
type: Playbook
title: Routing
description: How work is routed between agents.
---

# Routing

Claim before you route.
"#,
    );
    root
}

fn init_git(root: &std::path::Path) {
    let run = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "bagsy")
            .env("GIT_AUTHOR_EMAIL", "bagsy@example.com")
            .env("GIT_COMMITTER_NAME", "bagsy")
            .env("GIT_COMMITTER_EMAIL", "bagsy@example.com")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    };
    run(&["init", "-b", "main"]);
    run(&["add", "."]);
    run(&["commit", "-m", "init okf"]);
}

#[test]
fn get_prints_concept() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["get", "brain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Shared Brain"))
        .stdout(predicate::str::contains("type: Playbook"));
}

#[test]
fn claim_then_second_agent_collides() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);

    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["claim", "brain", "--agent", "agent-a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bagsied"));

    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["claim", "brain", "--agent", "agent-b", "--no-branch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already bagsied"));
}

#[test]
fn release_allows_reclaim() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);

    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["claim", "routing", "--agent", "agent-a", "--no-branch"])
        .assert()
        .success();

    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["release", "routing", "--agent", "agent-a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("released"));

    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["claim", "routing", "--agent", "agent-b", "--no-branch"])
        .assert()
        .success();
}

#[test]
fn propose_refuses_main() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);

    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["propose"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("never push main"));
}

#[test]
fn lint_passes_clean_bundle() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["lint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s)"));
}

#[test]
fn lint_fails_on_bad_frontmatter() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    write_concept(&root, "concepts/broken.md", "# no frontmatter\n");
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["lint"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("error:"));
}

#[test]
fn parse_frontmatter_unit() {
    // smoke via get on nested path
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    write_concept(
        &root,
        "concepts/nested/deep.md",
        "---\ntype: Reference\ntitle: Deep\n---\n\nBody.\n",
    );
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["get", "concepts/nested/deep"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deep"));
}
