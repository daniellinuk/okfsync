//! bagsy — Bagsy a concept so your agents don't clobber the brain.
//!
//! CLI for agents; `bagsy serve` holds the OKF KB. Per-agent bearer tokens
//! gate access. Workers claim, propose markdown, release. Never clone required.

mod api;
mod claim;
mod client;
mod config;
mod get;
mod git;
mod init;
mod lint;
mod lock;
mod okf;
mod propose;
mod release;
mod server;
mod token;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "bagsy",
    version,
    about = "Bagsy a concept so your agents don't clobber the brain.",
    long_about = "OSS CLI for agent-swarm shared memory on OKF.\n\
Agents talk to `bagsy serve` over HTTP with a per-agent bearer token.\n\
The KB owner runs init/serve/token on the data directory."
)]
struct Cli {
    /// Path to the OKF knowledge root (directory containing concepts/).
    #[arg(long, global = true, env = "BAGSY_ROOT")]
    root: Option<PathBuf>,

    /// Bagsy server URL (agent mode). Example: http://127.0.0.1:7432
    #[arg(long, global = true, env = "BAGSY_URL")]
    url: Option<String>,

    /// Bearer token issued by `bagsy token create` (agent mode).
    #[arg(long, global = true, env = "BAGSY_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new KB data directory (concepts/, .bagsy/, git).
    Init,
    /// Serve the KB over HTTP (owner). Default bind: 127.0.0.1:7432
    Serve {
        /// Listen address
        #[arg(long, default_value = "127.0.0.1:7432")]
        bind: String,
        /// `git push` after each successful propose (if origin exists)
        #[arg(long)]
        push: bool,
        /// Also push on this interval in seconds (0 = off)
        #[arg(long, default_value_t = 0)]
        push_interval: u64,
    },
    /// Create, list, or revoke per-agent tokens (owner; local data dir).
    Token {
        #[command(subcommand)]
        cmd: TokenCmd,
    },
    /// Read a concept (path relative to knowledge root, with or without .md).
    Get {
        /// Concept path, e.g. brain or concepts/brain.md
        concept: String,
    },
    /// Claim exclusive write access to a concept (server lock).
    Claim {
        /// Concept path to claim
        concept: String,
        /// Agent identity for local (no URL) mode. Ignored in server mode (token is identity).
        #[arg(long, env = "BAGSY_AGENT")]
        agent: Option<String>,
    },
    /// Release a claim on a concept (removes lock).
    Release {
        /// Concept path to release
        concept: String,
        /// Agent identity for local mode. Ignored in server mode.
        #[arg(long, env = "BAGSY_AGENT")]
        agent: Option<String>,
        /// Release even if another agent holds the lock
        #[arg(long)]
        force: bool,
    },
    /// Write a new markdown document for a claimed concept (server commits).
    Propose {
        /// Concept path
        concept: String,
        /// File containing the full markdown document
        #[arg(long)]
        file: PathBuf,
        /// Commit message title
        #[arg(long)]
        title: Option<String>,
        /// Push to origin after commit (local mode only; server uses serve --push)
        #[arg(long)]
        push: bool,
        /// Agent identity for local mode. Ignored in server mode.
        #[arg(long, env = "BAGSY_AGENT")]
        agent: Option<String>,
    },
    /// Lint OKF concepts + bagsy lock hygiene.
    Lint {
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TokenCmd {
    /// Issue a token for one agent (printed once).
    Create {
        #[arg(long)]
        agent: String,
        /// Replace any existing active token for this agent
        #[arg(long)]
        rotate: bool,
        /// Print JSON (id, agent, token) instead of text
        #[arg(long)]
        json: bool,
    },
    /// List tokens (hashes only; secrets are never stored).
    List,
    /// Revoke access for a token id or every token for an agent.
    Revoke {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        agent: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = config::resolve_root(cli.root.as_deref())?;
    let remote = client::from_opts(cli.url.as_deref(), cli.token.as_deref())?;

    match cli.command {
        Commands::Init => init::run(&root),
        Commands::Serve {
            bind,
            push,
            push_interval,
        } => {
            let addr: SocketAddr = bind
                .parse()
                .with_context(|| format!("invalid --bind '{bind}' (use host:port)"))?;
            server::run_blocking(root, addr, push, push_interval)
        }
        Commands::Token { cmd } => token_cmd(&root, cmd),
        Commands::Get { concept } => {
            if let Some(r) = remote {
                get::print_response(&r.get_concept(&concept)?);
                Ok(())
            } else {
                get::run(&root, &concept)
            }
        }
        Commands::Claim { concept, agent } => {
            if let Some(r) = remote {
                let resp = r.claim(&concept)?;
                println!("bagsied '{}'", resp.concept);
                println!("  agent: {}", resp.agent);
                println!();
                println!("Edit, then: bagsy propose {concept} --file <markdown>");
                Ok(())
            } else {
                let agent = config::default_agent(agent.as_deref());
                claim::run(&root, &concept, &agent)
            }
        }
        Commands::Release {
            concept,
            agent,
            force,
        } => {
            if let Some(r) = remote {
                let resp = r.release(&concept, force)?;
                println!(
                    "released '{}' (was held by {})",
                    resp.concept, resp.was_held_by
                );
                Ok(())
            } else {
                let agent = config::default_agent(agent.as_deref());
                release::run(&root, &concept, &agent, force)
            }
        }
        Commands::Propose {
            concept,
            file,
            title,
            push,
            agent,
        } => {
            let markdown = fs::read_to_string(&file)
                .with_context(|| format!("reading {}", file.display()))?;
            if let Some(r) = remote {
                let resp = r.propose(&concept, &markdown, title.as_deref())?;
                println!("proposed '{}'", resp.concept);
                println!("  agent:     {}", resp.agent);
                println!("  committed: {}", resp.committed);
                println!("  pushed:    {}", resp.pushed);
                Ok(())
            } else {
                let agent = config::default_agent(agent.as_deref());
                let out = propose::run(
                    &root,
                    &concept,
                    &agent,
                    &markdown,
                    title.as_deref(),
                    push,
                )?;
                propose::print_outcome(&out, &agent);
                Ok(())
            }
        }
        Commands::Lint { strict } => {
            if let Some(r) = remote {
                let report = r.lint(strict)?;
                lint::print_report(&report);
                if lint::failed(&report, strict) {
                    std::process::exit(1);
                }
                Ok(())
            } else {
                lint::run(&root, strict)
            }
        }
    }
}

fn token_cmd(root: &Path, cmd: TokenCmd) -> Result<()> {
    match cmd {
        TokenCmd::Create {
            agent,
            rotate,
            json,
        } => {
            let issued = token::create(root, &agent, rotate)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&issued).context("json token")?
                );
            } else {
                println!("created token for agent '{}'", issued.agent);
                println!("  id:    {}", issued.id);
                println!("  token: {}", issued.token);
                println!();
                println!("Store this token; bagsy will not show it again.");
                println!("Agent env:");
                println!("  export BAGSY_URL=http://127.0.0.1:7432");
                println!("  export BAGSY_TOKEN={}", issued.token);
            }
            Ok(())
        }
        TokenCmd::List => {
            let tokens = token::list(root)?;
            if tokens.is_empty() {
                println!("no tokens");
                return Ok(());
            }
            println!("{:<18} {:<20} {:<10} created", "id", "agent", "status");
            for t in tokens {
                let status = if t.is_active() { "active" } else { "revoked" };
                println!(
                    "{:<18} {:<20} {:<10} {}",
                    t.id,
                    t.agent,
                    status,
                    t.created_at.to_rfc3339()
                );
            }
            Ok(())
        }
        TokenCmd::Revoke { id, agent } => match (id, agent) {
            (Some(id), None) => {
                let rec = token::revoke_by_id(root, &id)?;
                println!("revoked token {} (agent {})", rec.id, rec.agent);
                Ok(())
            }
            (None, Some(agent)) => {
                let recs = token::revoke_by_agent(root, &agent)?;
                for rec in recs {
                    println!("revoked token {} (agent {})", rec.id, rec.agent);
                }
                Ok(())
            }
            _ => bail!("pass exactly one of --id <token-id> or --agent <name>"),
        },
    }
}
