use assert_cmd::cargo::cargo_bin;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn write_concept(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn init_okf(tmp: &TempDir) -> PathBuf {
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

fn init_git(root: &Path) {
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

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct Serve {
    child: Child,
    pub url: String,
}

impl Drop for Serve {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_serve(root: &Path) -> Serve {
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let url = format!("http://{bind}");
    let bin = cargo_bin("bagsy");
    let mut child = Command::new(&bin)
        .args(["serve", "--root"])
        .arg(root)
        .args(["--bind", &bind])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn serve");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            break;
        }
        if let Ok(Some(status)) = child.try_wait() {
            let mut err = String::new();
            if let Some(mut s) = child.stderr.take() {
                use std::io::Read;
                let _ = s.read_to_string(&mut err);
            }
            panic!("serve exited {status}: {err}");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("serve did not bind {bind} in time");
        }
        thread::sleep(Duration::from_millis(20));
    }
    Serve { child, url }
}

fn mint(root: &Path, agent: &str) -> String {
    let out = cargo_bin_cmd!("bagsy")
        .current_dir(root)
        .args(["token", "create", "--agent", agent, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["token"].as_str().unwrap().to_string()
}

#[test]
fn get_prints_concept_locally() {
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

#[test]
fn token_create_list_revoke() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let token = mint(&root, "alice");
    assert!(token.starts_with("bgy_"));
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["token", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("active"));
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["token", "revoke", "--agent", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("revoked"));
}

#[test]
fn server_two_agents_collide_then_recover() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);
    let tok_a = mint(&root, "agent-a");
    let tok_b = mint(&root, "agent-b");
    let serve = start_serve(&root);

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_a)
        .args(["claim", "brain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bagsied"));

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_b)
        .args(["claim", "brain"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already bagsied"));

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_a)
        .args(["release", "brain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("released"));

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_b)
        .args(["claim", "brain"])
        .assert()
        .success();
}

#[test]
fn server_rejects_revoked_token() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let tok = mint(&root, "ghost");
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["token", "revoke", "--agent", "ghost"])
        .assert()
        .success();
    let serve = start_serve(&root);
    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["get", "brain"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("revoked"));
}

#[test]
fn propose_requires_claim_then_writes() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);
    let tok = mint(&root, "writer");
    let serve = start_serve(&root);
    let md = root.join("new-brain.md");
    fs::write(
        &md,
        "---\ntype: Playbook\ntitle: Shared Brain\n---\n\nUpdated by writer.\n",
    )
    .unwrap();

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["propose", "brain", "--file"])
        .arg(&md)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not claimed"));

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["claim", "brain"])
        .assert()
        .success();

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["propose", "brain", "--file"])
        .arg(&md)
        .assert()
        .success()
        .stdout(predicate::str::contains("proposed"));

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["get", "brain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated by writer"));
}
