use anyhow::Result;
use std::path::Path;

use crate::api::LintResponse;
use crate::config::Config;
use crate::lock;
use crate::okf;

pub fn collect(root: &Path) -> Result<LintResponse> {
    let cfg = Config::load(root)?;
    let mut out = LintResponse::default();

    let concepts = okf::list_concepts(root)?;
    if concepts.is_empty() {
        out.warnings.push("no concepts/*.md files found".into());
    }
    for path in &concepts {
        let rel = okf::rel_from_root(root, path);
        match okf::read_concept(root, &rel) {
            Ok(c) => {
                if c.frontmatter.title.is_none() {
                    out.warnings.push(format!("{rel}: missing optional title"));
                }
            }
            Err(e) => out.errors.push(format!("{rel}: {e}")),
        }
    }
    out.concepts = concepts.len();

    let locks = lock::list_locks(root, &cfg)?;
    out.locks = locks.len();
    for (path, lk) in &locks {
        let concept_path = root.join(&lk.concept);
        if !concept_path.exists() {
            out.errors.push(format!(
                "lock {} points at missing concept {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                lk.concept
            ));
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    for (_, lk) in &locks {
        if !seen.insert(lk.concept.clone()) {
            out.errors.push(format!("duplicate lock for concept {}", lk.concept));
        }
    }

    Ok(out)
}

pub fn print_report(report: &LintResponse) {
    for w in &report.warnings {
        println!("warning: {w}");
    }
    for e in &report.errors {
        println!("error: {e}");
    }
    println!(
        "bagsy lint: {} concept(s), {} lock(s), {} error(s), {} warning(s)",
        report.concepts,
        report.locks,
        report.errors.len(),
        report.warnings.len()
    );
}

pub fn failed(report: &LintResponse, strict: bool) -> bool {
    !report.errors.is_empty() || (strict && !report.warnings.is_empty())
}

pub fn run(root: &Path, strict: bool) -> Result<()> {
    let report = collect(root)?;
    print_report(&report);
    if failed(&report, strict) {
        std::process::exit(1);
    }
    Ok(())
}
