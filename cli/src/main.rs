//! bagsy — Bagsy a concept so your agents don't clobber the brain.
//!
//! Collision hygiene for agent-swarm shared memory on git/OKF.
//! Workers claim concepts, write on a branch, propose a PR/MR, and lint.
//! Never push main.

mod claim;
mod config;
mod get;
mod git;
mod lint;
mod lock;
mod okf;
mod propose;
mod release;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "bagsy",
    version,
    about = "Bagsy a concept so your agents don't clobber the brain.",
    long_about = "OSS CLI for agent-swarm shared memory on git/OKF — collision hygiene.\n\
Workers bagsy/claim a concept, write on a branch, PR/MR, lint; never push main."
)]
struct Cli {
    /// Path to the OKF knowledge root (directory containing concepts/).
    #[arg(long, global = true, env = "BAGSY_ROOT")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Read a concept (path relative to knowledge root, with or without .md).
    Get {
        /// Concept path, e.g. concepts/brain or concepts/brain.md
        concept: String,
    },
    /// Claim exclusive write access to a concept (creates lock + branch).
    Claim {
        /// Concept path to claim
        concept: String,
        /// Agent identity (defaults to BAGSY_AGENT or $USER)
        #[arg(long, env = "BAGSY_AGENT")]
        agent: Option<String>,
        /// Skip creating/checking out a git branch
        #[arg(long)]
        no_branch: bool,
    },
    /// Release a claim on a concept (removes lock).
    Release {
        /// Concept path to release
        concept: String,
        /// Agent identity; must match the lock holder unless --force
        #[arg(long, env = "BAGSY_AGENT")]
        agent: Option<String>,
        /// Release even if another agent holds the lock
        #[arg(long)]
        force: bool,
    },
    /// Propose changes: refuse main, push the bagsy branch, print PR/MR hints.
    Propose {
        /// Optional title for the PR/MR hint
        #[arg(long)]
        title: Option<String>,
        /// Push with --force-with-lease (still never to main/master)
        #[arg(long)]
        force_with_lease: bool,
    },
    /// Lint OKF concepts + bagsy lock hygiene.
    Lint {
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = config::resolve_root(cli.root.as_deref())?;

    match cli.command {
        Commands::Get { concept } => get::run(&root, &concept),
        Commands::Claim {
            concept,
            agent,
            no_branch,
        } => claim::run(&root, &concept, agent.as_deref(), no_branch),
        Commands::Release {
            concept,
            agent,
            force,
        } => release::run(&root, &concept, agent.as_deref(), force),
        Commands::Propose {
            title,
            force_with_lease,
        } => propose::run(&root, title.as_deref(), force_with_lease),
        Commands::Lint { strict } => lint::run(&root, strict),
    }
}
