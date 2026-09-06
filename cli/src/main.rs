//! bagsy — Bagsy a concept so your agents don't clobber the brain.
//!
//! CLI for agents; `bagsy serve` holds the OKF wiki. Per-agent bearer tokens
//! gate access. Agents get and propose markdown. No claim/release. No deletes.

mod api;
mod client;
mod config;
mod get;
mod git;
mod init;
mod lint;
mod okf;
mod propose;
mod server;
mod token;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::fs;
use std::io::{IsTerminal, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "bagsy",
    version,
    about = "Bagsy a concept so your agents don't clobber the brain.",
    long_about = "OSS CLI for a multi-agent OKF wiki.\n\
Agents talk to `bagsy serve` over HTTP with a per-agent bearer token.\n\
The CLI can read and propose (create/update) concepts. It cannot delete.\n\
The KB owner runs init/serve/token on the data directory. Gardening is out of band.",
    after_help = "Examples:
  bagsy get --help
  bagsy propose --help
  bagsy lint --help
  bagsy token --help
  bagsy init --help
  bagsy serve --help"
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
    #[command(after_help = "Examples:
  bagsy init --root ./my-kb
  bagsy init --root ./my-kb --json")]
    Init {
        /// Machine-readable JSON on stdout
        #[arg(long)]
        json: bool,
    },
    /// Serve the KB over HTTP (owner). Default bind: 127.0.0.1:7432
    #[command(after_help = "Examples:
  bagsy serve --root ./my-kb
  bagsy serve --root ./my-kb --bind 127.0.0.1:7432
  bagsy serve --root ./my-kb --bind 127.0.0.1:0 --json")]
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
        /// Print listen URL as JSON, then serve
        #[arg(long)]
        json: bool,
    },
    /// Create, list, or revoke per-agent tokens (owner; local data dir).
    #[command(after_help = "Examples:
  bagsy token create --help
  bagsy token list --help
  bagsy token revoke --help")]
    Token {
        #[command(subcommand)]
        cmd: TokenCmd,
    },
    /// Read a concept (raw markdown, round-trips into propose).
    #[command(after_help = "Examples:
  bagsy get brain
  bagsy get brain --json
  bagsy get brain --url http://127.0.0.1:7432 --token \"$BAGSY_TOKEN\"
  bagsy get brain > /tmp/brain.md")]
    Get {
        /// Concept path, e.g. brain or concepts/brain.md
        concept: String,
        /// Machine-readable JSON (includes `markdown` for propose)
        #[arg(long)]
        json: bool,
    },
    /// Create or update a concept (never deletes). Server commits on its clone.
    #[command(after_help = "Examples:
  bagsy propose brain --file ./brain.md
  bagsy propose brain --file -
  cat brain.md | bagsy propose brain --file -
  bagsy propose brain --file ./brain.md --dry-run
  bagsy propose brain --file ./brain.md --json")]
    Propose {
        /// Concept path
        concept: String,
        /// Markdown file, or `-` for stdin
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,
        /// Commit message title
        #[arg(long)]
        title: Option<String>,
        /// Push to origin after commit (local mode only; server uses serve --push)
        #[arg(long)]
        push: bool,
        /// Agent identity for local mode. Ignored in server mode (token is identity).
        #[arg(long, env = "BAGSY_AGENT")]
        agent: Option<String>,
        /// Validate path + frontmatter; do not write
        #[arg(long)]
        dry_run: bool,
        /// Machine-readable JSON on stdout
        #[arg(long)]
        json: bool,
    },
    /// Lint OKF concepts (frontmatter).
    #[command(after_help = "Examples:
  bagsy lint
  bagsy lint --strict
  bagsy lint --json")]
    Lint {
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
        /// Machine-readable JSON on stdout
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TokenCmd {
    /// Issue a token for one agent (printed once).
    #[command(after_help = "Examples:
  bagsy token create --agent worker-1
  bagsy token create --agent worker-1 --json
  bagsy token create --agent worker-1 --rotate --json")]
    Create {
        /// Agent id (one active token unless --rotate)
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
    #[command(after_help = "Examples:
  bagsy token list
  bagsy token list --json")]
    List {
        /// Machine-readable JSON on stdout
        #[arg(long)]
        json: bool,
    },
    /// Revoke access for a token id or every token for an agent.
    #[command(after_help = "Examples:
  bagsy token revoke --agent worker-1
  bagsy token revoke --id <token-id>
  bagsy token revoke --agent worker-1 --dry-run
  bagsy token list")]
    Revoke {
        #[arg(long)]
        id: Option<String>,
        /// Agent name (revokes every active token for that agent)
        #[arg(long)]
        agent: Option<String>,
        /// Show what would be revoked; do not write
        #[arg(long)]
        dry_run: bool,
        /// Machine-readable JSON on stdout
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = config::resolve_root(cli.root.as_deref())?;
    let remote = client::from_opts(cli.url.as_deref(), cli.token.as_deref())?;

    match cli.command {
        Commands::Init { json } => init::run(&root, json),
        Commands::Serve {
            bind,
            push,
            push_interval,
            json,
        } => {
            let addr: SocketAddr = bind.parse().with_context(|| {
                format!(
                    "invalid --bind '{bind}' (use host:port)\n  bagsy serve --bind 127.0.0.1:7432"
                )
            })?;
            server::run_blocking(root, addr, push, push_interval, json)
        }
        Commands::Token { cmd } => token_cmd(&root, cmd),
        Commands::Get { concept, json } => {
            if let Some(r) = remote {
                get::print_response(&r.get_concept(&concept)?, json)
            } else {
                get::run(&root, &concept, json)
            }
        }
        Commands::Propose {
            concept,
            file,
            title,
            push,
            agent,
            dry_run,
            json,
        } => {
            let markdown = read_propose_markdown(file.as_deref())?;
            if dry_run {
                let rel = okf::validate_propose(&concept, &markdown)?;
                let agent = if remote.is_some() {
                    "token".to_string()
                } else {
                    config::default_agent(agent.as_deref())
                };
                return propose::print_outcome(
                    &propose::Outcome {
                        rel,
                        committed: false,
                        pushed: false,
                        dry_run: true,
                    },
                    &agent,
                    json,
                );
            }
            if let Some(r) = remote {
                let resp = r.propose(&concept, &markdown, title.as_deref())?;
                propose::print_outcome(
                    &propose::Outcome {
                        rel: resp.concept,
                        committed: resp.committed,
                        pushed: resp.pushed,
                        dry_run: false,
                    },
                    &resp.agent,
                    json,
                )
            } else {
                let agent = config::default_agent(agent.as_deref());
                let out = propose::run(
                    &root,
                    &concept,
                    &agent,
                    &markdown,
                    title.as_deref(),
                    push,
                    false,
                )?;
                propose::print_outcome(&out, &agent, json)
            }
        }
        Commands::Lint { strict, json } => {
            if let Some(r) = remote {
                let report = r.lint(strict)?;
                lint::print_report(&report, json)?;
                if lint::failed(&report, strict) {
                    std::process::exit(1);
                }
                Ok(())
            } else {
                lint::run(&root, strict, json)
            }
        }
    }
}

fn read_propose_markdown(file: Option<&Path>) -> Result<String> {
    match file {
        Some(p) if p.as_os_str() == "-" => read_stdin(),
        Some(p) => fs::read_to_string(p).with_context(|| {
            format!(
                "reading {}\n  bagsy propose <concept> --file <path.md>",
                p.display()
            )
        }),
        None => {
            if std::io::stdin().is_terminal() {
                bail!(
                    "markdown required\n  bagsy propose <concept> --file <path.md>\n  bagsy propose <concept> --file -\n  cat page.md | bagsy propose <concept> --file -"
                );
            }
            read_stdin()
        }
    }
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("reading stdin")?;
    if buf.trim().is_empty() {
        bail!(
            "stdin was empty\n  bagsy propose <concept> --file <path.md>\n  cat page.md | bagsy propose <concept> --file -"
        );
    }
    Ok(buf)
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
                println!("{}", serde_json::to_string(&issued).context("json token")?);
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
        TokenCmd::List { json } => {
            let tokens = token::list(root)?;
            if json {
                let rows: Vec<TokenListJson> = tokens.iter().map(TokenListJson::from).collect();
                println!("{}", serde_json::to_string(&rows).context("json list")?);
                return Ok(());
            }
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
        TokenCmd::Revoke {
            id,
            agent,
            dry_run,
            json,
        } => match (id, agent) {
            (Some(id), None) => {
                let (rec, changed) = token::revoke_by_id(root, &id, dry_run)?;
                print_revoke(json, dry_run, changed, &[rec])
            }
            (None, Some(agent)) => {
                let (recs, changed) = token::revoke_by_agent(root, &agent, dry_run)?;
                print_revoke(json, dry_run, changed, &recs)
            }
            _ => bail!(
                "pass exactly one of --id or --agent\n  bagsy token revoke --agent <name>\n  bagsy token revoke --id <token-id>\n  bagsy token list"
            ),
        },
    }
}

#[derive(Serialize)]
struct TokenListJson {
    id: String,
    agent: String,
    status: String,
    created_at: String,
    revoked_at: Option<String>,
}

impl From<&token::TokenRecord> for TokenListJson {
    fn from(t: &token::TokenRecord) -> Self {
        Self {
            id: t.id.clone(),
            agent: t.agent.clone(),
            status: if t.is_active() {
                "active".into()
            } else {
                "revoked".into()
            },
            created_at: t.created_at.to_rfc3339(),
            revoked_at: t.revoked_at.map(|d| d.to_rfc3339()),
        }
    }
}

#[derive(Serialize)]
struct RevokeJson {
    dry_run: bool,
    changed: bool,
    tokens: Vec<TokenListJson>,
}

fn print_revoke(
    json: bool,
    dry_run: bool,
    changed: bool,
    recs: &[token::TokenRecord],
) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(&RevokeJson {
                dry_run,
                changed,
                tokens: recs.iter().map(TokenListJson::from).collect(),
            })
            .context("json revoke")?
        );
        return Ok(());
    }
    if recs.is_empty() {
        println!("already revoked");
        return Ok(());
    }
    for rec in recs {
        if dry_run {
            println!("would revoke token {} (agent {})", rec.id, rec.agent);
        } else if changed {
            println!("revoked token {} (agent {})", rec.id, rec.agent);
        } else {
            println!("already revoked token {} (agent {})", rec.id, rec.agent);
        }
    }
    Ok(())
}
