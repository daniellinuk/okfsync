use anyhow::{Context, Result};
use std::path::Path;

use crate::api::LintResponse;
use crate::okf;

pub fn collect(root: &Path) -> Result<LintResponse> {
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
    Ok(out)
}

pub fn print_report(report: &LintResponse, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(report).context("json lint")?);
        return Ok(());
    }
    for w in &report.warnings {
        println!("warning: {w}");
    }
    for e in &report.errors {
        println!("error: {e}");
    }
    println!(
        "kbsync lint: {} concept(s), {} error(s), {} warning(s)",
        report.concepts,
        report.errors.len(),
        report.warnings.len()
    );
    Ok(())
}

pub fn failed(report: &LintResponse, strict: bool) -> bool {
    !report.errors.is_empty() || (strict && !report.warnings.is_empty())
}

pub fn run(root: &Path, strict: bool, json: bool) -> Result<()> {
    let report = collect(root)?;
    print_report(&report, json)?;
    if failed(&report, strict) {
        std::process::exit(1);
    }
    Ok(())
}
