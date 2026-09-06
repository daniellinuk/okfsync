use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::git;
use crate::lock;
use crate::okf;

#[derive(Debug, Default)]
struct Report {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl Report {
    fn err(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
    fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

pub fn run(root: &Path, strict: bool) -> Result<()> {
    let cfg = Config::load(root)?;
    let mut report = Report::default();

    // 1) OKF concepts parse
    let concepts = okf::list_concepts(root)?;
    if concepts.is_empty() {
        report.warn("no concepts/*.md files found");
    }
    for path in &concepts {
        let rel = okf::rel_from_root(root, path);
        match okf::read_concept(root, &rel) {
            Ok(c) => {
                if c.frontmatter.title.is_none() {
                    report.warn(format!("{rel}: missing optional title"));
                }
            }
            Err(e) => report.err(format!("{rel}: {e}")),
        }
    }

    // 2) Lock hygiene
    let locks = lock::list_locks(root, &cfg)?;
    for (path, lk) in &locks {
        let concept_path = root.join(&lk.concept);
        if !concept_path.exists() {
            report.err(format!(
                "lock {} points at missing concept {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                lk.concept
            ));
        }
        if cfg.is_protected_branch(&lk.branch) {
            report.err(format!(
                "lock for {} claims protected branch '{}'",
                lk.concept, lk.branch
            ));
        }
        if !lk.branch.starts_with("bagsy/") {
            report.warn(format!(
                "lock for {} uses non-bagsy branch '{}'",
                lk.concept, lk.branch
            ));
        }
    }

    // Detect duplicate locks for same concept (shouldn't happen with path scheme)
    let mut seen = std::collections::BTreeSet::new();
    for (_, lk) in &locks {
        if !seen.insert(lk.concept.clone()) {
            report.err(format!("duplicate lock for concept {}", lk.concept));
        }
    }

    // 3) Never-push-main: if on protected branch with bagsy locks staged oddly, warn;
    //    if HEAD is protected and dirty with concept edits while locks exist for other agents — soft.
    if let Some(git_root) = git::find_git_toplevel(root) {
        if git::is_git_repo(&git_root) {
            if let Ok(branch) = git::current_branch(&git_root) {
                if cfg.is_protected_branch(&branch) && !locks.is_empty() {
                    report.warn(format!(
                        "on protected branch '{branch}' with {} active lock(s) — \
claims should live on bagsy/* branches, not main",
                        locks.len()
                    ));
                }
            }
        }
    }

    // 4) Collision signal: lock file claims agent A but concept file mtime... skip — keep MVP simple.

    for w in &report.warnings {
        println!("warning: {w}");
    }
    for e in &report.errors {
        println!("error: {e}");
    }

    let warn_n = report.warnings.len();
    let err_n = report.errors.len();
    println!(
        "bagsy lint: {} concept(s), {} lock(s), {err_n} error(s), {warn_n} warning(s)",
        concepts.len(),
        locks.len()
    );

    if err_n > 0 || (strict && warn_n > 0) {
        std::process::exit(1);
    }
    Ok(())
}
