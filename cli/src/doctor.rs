//! `kbsync doctor` — one-shot diagnostics for agents and owners.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::api::{ConceptSummary, DoctorResponse, HealthResponse};
use crate::client;
use crate::config;
use crate::git;
use crate::lint;
use crate::okf;
use crate::token;

#[derive(Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub version: String,
    pub binary: Option<String>,
    pub mode: String,
    pub url: Option<String>,
    pub reachable: Option<bool>,
    pub root: Option<String>,
    pub agent: Option<String>,
    pub token_id: Option<String>,
    pub token_fingerprint: Option<String>,
    pub token_source: String,
    pub token_file: Option<String>,
    pub token_file_mode: Option<String>,
    pub concepts: Option<usize>,
    pub by_type: BTreeMap<String, usize>,
    pub lint_errors: Option<usize>,
    pub lint_warnings: Option<usize>,
    pub git_repo: Option<bool>,
    pub git_branch: Option<String>,
    pub git_origin: Option<bool>,
    pub tokens_active: Option<usize>,
    pub bun_bin_on_path: Option<bool>,
    pub kbsync_on_path: bool,
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn run(
    root: &Path,
    json: bool,
    url: Option<&str>,
    token: Option<&str>,
    token_file: Option<&Path>,
) -> Result<()> {
    let report = collect(root, url, token, token_file)?;
    print_report(&report, json)?;
    if !report.errors.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn collect(
    root: &Path,
    url: Option<&str>,
    token: Option<&str>,
    token_file: Option<&Path>,
) -> Result<DoctorReport> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut notes = Vec::new();

    let binary = std::env::current_exe()
        .ok()
        .map(|p| p.display().to_string());
    let kbsync_on_path = command_on_path("kbsync");
    if !kbsync_on_path {
        warnings.push(
            "kbsync is not on PATH in this environment\n  bun add -g okfsync\n  export PATH=\"$HOME/.bun/bin:$PATH\""
                .into(),
        );
    }
    let bun_bin_on_path = bun_bin_dir().and_then(|d| {
        if !d.is_dir() {
            return None;
        }
        let on = dir_on_path(&d);
        if !on {
            warnings.push(format!(
                "{} exists but is not on PATH\n  export PATH=\"{}:$PATH\"",
                d.display(),
                d.display()
            ));
        }
        Some(on)
    });

    let (token_source, token_file_path) = token_source(token, token_file);
    if token_source == "env" && token_file_path.is_some() {
        warnings.push(
            "KBSYNC_TOKEN is set; KBSYNC_TOKEN_FILE is ignored\n  unset KBSYNC_TOKEN to use the file"
                .into(),
        );
    }
    let token_file_mode = token_file_path.as_ref().and_then(|p| unix_mode(p));
    if let Some(path) = &token_file_path {
        if !path.is_file() {
            errors.push(format!(
                "token file not found: {}\n  export KBSYNC_TOKEN_FILE=/path/to/agent.token",
                path.display()
            ));
        } else if let Some(mode) = &token_file_mode {
            if file_mode_too_open(path) {
                warnings.push(format!(
                    "{} is readable by group/other (mode {mode})\n  chmod 600 {}",
                    path.display(),
                    path.display()
                ));
            }
        }
    }

    let mut report = DoctorReport {
        ok: false,
        version: env!("CARGO_PKG_VERSION").into(),
        binary,
        mode: "local".into(),
        url: None,
        reachable: None,
        root: None,
        agent: None,
        token_id: None,
        token_fingerprint: None,
        token_source: token_source.to_string(),
        token_file: token_file_path.as_ref().map(|p| p.display().to_string()),
        token_file_mode,
        concepts: None,
        by_type: BTreeMap::new(),
        lint_errors: None,
        lint_warnings: None,
        git_repo: None,
        git_branch: None,
        git_origin: None,
        tokens_active: None,
        bun_bin_on_path,
        kbsync_on_path,
        notes: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    match client::from_opts(url, token, token_file) {
        Ok(Some(remote)) => {
            report.mode = "remote".into();
            report.url = Some(remote.url.clone());
            match client::probe_health(&remote.url) {
                Ok(h) => apply_health(&mut report, &h, &mut errors),
                Err(e) => {
                    report.reachable = Some(false);
                    errors.push(format!(
                        "cannot reach okfsync server: {e}\n  kbsync serve --root <data-dir>\n  export KBSYNC_URL=http://127.0.0.1:7432"
                    ));
                }
            }
            if report.reachable == Some(true) {
                match remote.doctor() {
                    Ok(d) => apply_server_doctor(&mut report, &remote.token, d, &mut notes),
                    Err(e) => errors.push(format!(
                        "doctor/whoami failed: {e}\n  kbsync whoami --json\n  kbsync token create --agent <id>"
                    )),
                }
            }
        }
        Ok(None) => {
            report.mode = "local".into();
            report.root = Some(root.display().to_string());
            fill_local(root, &mut report, &mut errors, &mut warnings, &mut notes);
        }
        Err(e) => {
            let msg = e.to_string();
            errors.push(msg);
            if let Some(u) = nonempty(url).or_else(|| nonempty_env("KBSYNC_URL")) {
                report.mode = "remote".into();
                report.url = Some(u.clone());
                match client::probe_health(&u) {
                    Ok(h) => apply_health(&mut report, &h, &mut errors),
                    Err(he) => {
                        report.reachable = Some(false);
                        errors.push(format!("cannot reach okfsync server: {he}"));
                    }
                }
            } else {
                report.mode = "local".into();
                report.root = Some(root.display().to_string());
                fill_local(root, &mut report, &mut errors, &mut warnings, &mut notes);
            }
        }
    }

    report.warnings = warnings;
    report.errors = errors;
    report.notes = notes;
    report.ok = report.errors.is_empty();
    Ok(report)
}

fn apply_health(report: &mut DoctorReport, h: &HealthResponse, errors: &mut Vec<String>) {
    report.reachable = Some(h.ok);
    if !h.ok {
        errors.push("server /health returned ok=false".into());
    }
}

fn apply_server_doctor(
    report: &mut DoctorReport,
    presented_token: &str,
    d: DoctorResponse,
    notes: &mut Vec<String>,
) {
    report.agent = Some(d.agent);
    report.token_id = Some(d.token_id);
    report.token_fingerprint = Some(token::fingerprint(presented_token));
    report.concepts = Some(d.concepts);
    report.by_type = d.by_type;
    report.lint_errors = Some(d.lint_errors);
    report.lint_warnings = Some(d.lint_warnings);
    report.git_repo = Some(d.git_repo);
    report.git_branch = d.git_branch;
    report.git_origin = Some(d.git_origin);
    report.tokens_active = Some(d.tokens_active);
    push_git_notes(notes, d.git_repo, d.git_origin);
    if d.concepts == 0 {
        notes.push("0 concepts\n  kbsync propose brain --file ./brain.md".into());
    }
}

fn fill_local(
    root: &Path,
    report: &mut DoctorReport,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    notes: &mut Vec<String>,
) {
    if !config::looks_like_kb(root) {
        errors.push(format!(
            "{} does not look like an okfsync KB (need concepts/ or .okfsync/)\n  kbsync init --root {}\n  export KBSYNC_URL=http://127.0.0.1:7432",
            root.display(),
            root.display()
        ));
        return;
    }
    match okf::summaries(root) {
        Ok(pages) => {
            report.concepts = Some(pages.len());
            report.by_type = count_types(&pages);
            if pages.is_empty() {
                notes.push("0 concepts\n  kbsync propose brain --file ./brain.md".into());
            }
        }
        Err(e) => errors.push(format!("cannot list concepts: {e}")),
    }
    match lint::collect(root) {
        Ok(l) => {
            report.lint_errors = Some(l.errors.len());
            report.lint_warnings = Some(l.warnings.len());
        }
        Err(e) => warnings.push(format!("lint failed: {e}")),
    }
    let repo = git::is_git_repo(root);
    report.git_repo = Some(repo);
    if repo {
        report.git_branch = git::current_branch(root).ok();
        report.git_origin = Some(git::has_remote(root, "origin"));
        push_git_notes(notes, true, report.git_origin.unwrap_or(false));
    } else {
        notes.push(
            "no git repo — propose writes files but committed:false\n  git is created by `kbsync init`"
                .into(),
        );
    }
    if let Ok(tokens) = token::list(root) {
        report.tokens_active = Some(tokens.iter().filter(|t| t.is_active()).count());
    }
}

fn push_git_notes(notes: &mut Vec<String>, git_repo: bool, git_origin: bool) {
    if !git_repo {
        return;
    }
    if git_origin {
        notes.push(
            "propose pushed:false is not a failure. Push only happens if the owner ran `kbsync serve --push` (and origin exists)."
                .into(),
        );
    } else {
        notes.push(
            "propose pushed:false is normal — no git origin. The page is committed on the server disk."
                .into(),
        );
    }
}

fn count_types(pages: &[ConceptSummary]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for c in pages {
        let key = if c.r#type.trim().is_empty() {
            "(none)".to_string()
        } else {
            c.r#type.clone()
        };
        *m.entry(key).or_insert(0) += 1;
    }
    m
}

fn token_source(token: Option<&str>, token_file: Option<&Path>) -> (&'static str, Option<PathBuf>) {
    let file = token_file
        .map(|p| p.to_path_buf())
        .or_else(|| nonempty_env("KBSYNC_TOKEN_FILE").map(PathBuf::from));
    if nonempty(token)
        .or_else(|| nonempty_env("KBSYNC_TOKEN"))
        .is_some()
    {
        ("env", file)
    } else if file.is_some() {
        ("file", file)
    } else {
        ("none", file)
    }
}

fn nonempty(v: Option<&str>) -> Option<String> {
    v.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn nonempty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn bun_bin_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".bun").join("bin"))
}

fn path_dirs() -> Vec<PathBuf> {
    match std::env::var_os("PATH") {
        Some(p) => std::env::split_paths(&p).collect(),
        None => vec![],
    }
}

fn dir_on_path(dir: &Path) -> bool {
    let want = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    path_dirs().iter().any(|p| {
        let got = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        got == want
    })
}

fn command_on_path(name: &str) -> bool {
    path_dirs().iter().any(|dir| {
        let p = dir.join(name);
        p.is_file()
    })
}

fn unix_mode(path: &Path) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path).ok()?.permissions().mode() & 0o777;
        Some(format!("{mode:03o}"))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

fn file_mode_too_open(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o077 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

fn print_report(report: &DoctorReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(report).context("json doctor")?);
        return Ok(());
    }
    let status = if report.ok { "ok" } else { "not ok" };
    println!("kbsync doctor: {status}");
    println!("  version:    {}", report.version);
    if let Some(n) = report.concepts {
        println!("  concepts:   {n}");
        for (kind, n) in &report.by_type {
            println!("    {kind:<12} {n}");
        }
    }
    if let Some(bin) = &report.binary {
        println!("  binary:     {bin}");
    }
    println!("  mode:       {}", report.mode);
    if let Some(root) = &report.root {
        println!("  root:       {root}");
    }
    if let Some(url) = &report.url {
        let reach = match report.reachable {
            Some(true) => "reachable",
            Some(false) => "unreachable",
            None => "unknown",
        };
        println!("  url:        {url}  ({reach})");
    }
    if let Some(agent) = &report.agent {
        println!("  agent:      {agent}");
    }
    if let Some(id) = &report.token_id {
        let fp = report.token_fingerprint.as_deref().unwrap_or("-");
        println!(
            "  token:      id={id}  fingerprint={fp}  source={}",
            report.token_source
        );
    } else {
        println!("  token:      source={}", report.token_source);
    }
    if let Some(path) = &report.token_file {
        let mode = report.token_file_mode.as_deref().unwrap_or("-");
        println!("  token_file: {path}  mode={mode}");
    }
    if let (Some(e), Some(w)) = (report.lint_errors, report.lint_warnings) {
        println!("  lint:       {e} error(s), {w} warning(s)");
    }
    match report.git_repo {
        Some(true) => {
            let branch = report.git_branch.as_deref().unwrap_or("-");
            let origin = match report.git_origin {
                Some(true) => "yes",
                Some(false) => "no",
                None => "?",
            };
            println!("  git:        {branch}  origin={origin}");
        }
        Some(false) => println!("  git:        no"),
        None => {}
    }
    if let Some(n) = report.tokens_active {
        println!("  tokens:     {n} active");
    }
    let bun = match report.bun_bin_on_path {
        Some(true) => "yes",
        Some(false) => "no",
        None => "n/a",
    };
    println!(
        "  path:       kbsync={}  ~/.bun/bin={bun}",
        if report.kbsync_on_path { "yes" } else { "no" }
    );
    for n in &report.notes {
        println!("  note:       {}", n.replace('\n', "\n              "));
    }
    for w in &report.warnings {
        println!("  warning:    {}", w.replace('\n', "\n              "));
    }
    for e in &report.errors {
        println!("  error:      {}", e.replace('\n', "\n              "));
    }
    Ok(())
}
