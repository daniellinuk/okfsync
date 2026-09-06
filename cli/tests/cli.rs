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
    fs::create_dir_all(root.join(".bagsy")).unwrap();
    fs::write(
        root.join(".bagsy/config.toml"),
        "default_branch = \"main\"\n",
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
        .stdout(predicate::str::starts_with("---"))
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
fn help_includes_examples() {
    for args in [
        vec!["--help"],
        vec!["list", "--help"],
        vec!["search", "--help"],
        vec!["get", "--help"],
        vec!["propose", "--help"],
        vec!["lint", "--help"],
        vec!["token", "create", "--help"],
        vec!["token", "revoke", "--help"],
        vec!["init", "--help"],
        vec!["serve", "--help"],
    ] {
        cargo_bin_cmd!("bagsy")
            .args(&args)
            .assert()
            .success()
            .stdout(predicate::str::contains("Examples:"));
    }
}

#[test]
fn get_round_trips_into_propose() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);
    let out = cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["get", "brain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let md = root.join("roundtrip.md");
    fs::write(&md, &out).unwrap();
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["propose", "brain", "--file"])
        .arg(&md)
        .assert()
        .success()
        .stdout(predicate::str::contains("unchanged"));
}

#[test]
fn propose_reads_stdin() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["propose", "brain", "--file", "-"])
        .write_stdin("---\ntype: Playbook\ntitle: Shared Brain\n---\n\nFrom stdin.\n")
        .assert()
        .success();
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["get", "brain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("From stdin."));
}

#[test]
fn get_json_includes_markdown() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let out = cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["get", "brain", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["rel"], "concepts/brain.md");
    assert!(v["markdown"].as_str().unwrap().starts_with("---"));
}

#[test]
fn missing_token_error_includes_invocation() {
    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", "http://127.0.0.1:9")
        .args(["get", "brain"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("bagsy get"))
        .stderr(predicate::str::contains("--token"));
}

#[test]
fn token_revoke_without_target_includes_invocation() {
    cargo_bin_cmd!("bagsy")
        .args(["token", "revoke"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("token revoke --agent"));
}

#[test]
fn propose_dry_run_does_not_write() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let original = fs::read_to_string(root.join("concepts/brain.md")).unwrap();
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["propose", "brain", "--file", "-", "--dry-run"])
        .write_stdin("---\ntype: Playbook\ntitle: Shared Brain\n---\n\nWould not save.\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"));
    assert_eq!(
        fs::read_to_string(root.join("concepts/brain.md")).unwrap(),
        original
    );
}

#[test]
fn token_revoke_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    mint(&root, "alice");
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["token", "revoke", "--agent", "alice"])
        .assert()
        .success();
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["token", "revoke", "--agent", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already revoked"));
}

#[test]
fn get_without_concept_includes_invocation() {
    cargo_bin_cmd!("bagsy")
        .args(["get"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("bagsy get brain"));
}

#[test]
fn token_create_without_agent_includes_invocation() {
    cargo_bin_cmd!("bagsy")
        .args(["token", "create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("token create --agent"));
}

#[test]
fn url_token_flags_are_agent_only() {
    cargo_bin_cmd!("bagsy")
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--root"))
        .stdout(predicate::str::contains("--url").not());
    cargo_bin_cmd!("bagsy")
        .args(["serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--url").not());
    cargo_bin_cmd!("bagsy")
        .args(["token", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--url").not());
    cargo_bin_cmd!("bagsy")
        .args(["get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--token"));
}

#[test]
fn init_ignores_agent_url_env() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", "http://127.0.0.1:9")
        .args(["init", "--root"])
        .arg(root)
        .assert()
        .success();
    assert!(root.join("concepts").is_dir());
}

#[test]
fn list_and_search_concepts() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("concepts/brain.md"))
        .stdout(predicate::str::contains("concepts/routing.md"));
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["search", "routing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("concepts/routing.md"))
        .stdout(predicate::str::contains("concepts/brain.md").not());
    let out = cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert!(v["concepts"].as_array().unwrap().len() >= 2);
}

#[test]
fn search_without_query_includes_invocation() {
    cargo_bin_cmd!("bagsy")
        .args(["search"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("bagsy search routing"));
}

#[test]
fn server_list_and_search() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let tok = mint(&root, "reader");
    let serve = start_serve(&root);
    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("concepts/brain.md"));
    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok)
        .args(["search", "Routing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("concepts/routing.md"));
}

#[test]
fn claim_release_delete_gardener_are_not_commands() {
    for cmd in ["claim", "release", "delete", "gardener"] {
        cargo_bin_cmd!("bagsy")
            .args([cmd, "brain"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));
    }
}

#[test]
fn server_propose_updates_without_claim() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);
    let tok_a = mint(&root, "agent-a");
    let tok_b = mint(&root, "agent-b");
    let serve = start_serve(&root);
    let md = root.join("new-brain.md");
    fs::write(
        &md,
        "---\ntype: Playbook\ntitle: Shared Brain\n---\n\nUpdated by A.\n",
    )
    .unwrap();

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_a)
        .args(["propose", "brain", "--file"])
        .arg(&md)
        .assert()
        .success()
        .stdout(predicate::str::contains("proposed"));

    fs::write(
        &md,
        "---\ntype: Playbook\ntitle: Shared Brain\n---\n\nUpdated by B.\n",
    )
    .unwrap();
    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_b)
        .args(["propose", "brain", "--file"])
        .arg(&md)
        .assert()
        .success();

    cargo_bin_cmd!("bagsy")
        .env("BAGSY_URL", &serve.url)
        .env("BAGSY_TOKEN", &tok_b)
        .args(["get", "brain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated by B"));
}

#[test]
fn propose_refuses_path_traversal() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let md = root.join("evil.md");
    fs::write(&md, "---\ntype: Playbook\ntitle: X\n---\n\nnope\n").unwrap();
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["propose", "--file"])
        .arg(&md)
        .args(["../secrets"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing").or(predicate::str::contains("must be")));
}

fn http_exchange(method: &str, url: &str, token: &str) -> (u16, String) {
    use std::io::{Read, Write};
    let rest = url.trim_start_matches("http://");
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let path = if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{path}")
    };
    let mut stream = std::net::TcpStream::connect(host).unwrap();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).unwrap();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    let status = buf
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, buf)
}

#[test]
fn server_rejects_http_delete() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    let tok = mint(&root, "reader");
    let serve = start_serve(&root);
    let (status, body) = http_exchange(
        "DELETE",
        &format!("{}/v1/concepts?path=brain", serve.url),
        &tok,
    );
    assert_eq!(status, 405, "body: {body}");
    assert!(
        body.to_lowercase().contains("cannot delete") || body.contains("gardening"),
        "body: {body}"
    );
    let (status2, body2) = http_exchange("DELETE", &format!("{}/v1/proposals", serve.url), &tok);
    assert_eq!(status2, 405, "body: {body2}");
    assert!(root.join("concepts/brain.md").is_file());
}

#[test]
fn propose_does_not_delete_other_concepts() {
    let tmp = TempDir::new().unwrap();
    let root = init_okf(&tmp);
    init_git(&root);
    let md = root.join("brain.md");
    fs::write(
        &md,
        "---\ntype: Playbook\ntitle: Shared Brain\n---\n\nStill here.\n",
    )
    .unwrap();
    cargo_bin_cmd!("bagsy")
        .current_dir(&root)
        .args(["propose", "brain", "--file"])
        .arg(&md)
        .args(["--agent", "writer"])
        .assert()
        .success();
    assert!(root.join("concepts/brain.md").is_file());
    assert!(root.join("concepts/routing.md").is_file());
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
