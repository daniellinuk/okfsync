//! `bagsy serve` — HTTP API over one OKF data directory.

use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::api::{
    ClaimRequest, ClaimResponse, ConceptResponse, ErrorBody, HealthResponse, LintResponse,
    ProposeRequest, ProposeResponse, ReleaseRequest, ReleaseResponse, API_VERSION,
};
use crate::config::Config;
use crate::git;
use crate::lock;
use crate::okf;
use crate::token;

struct AppState {
    root: PathBuf,
    mutex: Mutex<()>,
    push: bool,
}

type ApiError = (StatusCode, Json<ErrorBody>);

fn err(status: StatusCode, msg: impl Into<String>) -> ApiError {
    (
        status,
        Json(ErrorBody {
            error: msg.into(),
        }),
    )
}

fn agent_from(headers: &HeaderMap, root: &std::path::Path) -> Result<String, ApiError> {
    let raw = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let presented = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .unwrap_or(raw)
        .trim();
    if presented.is_empty() {
        if let Some(alt) = headers.get("x-bagsy-token").and_then(|v| v.to_str().ok()) {
            return token::authenticate(root, alt.trim()).map_err(|e| err(StatusCode::UNAUTHORIZED, e.to_string()));
        }
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "missing bearer token (Authorization: Bearer … or BAGSY_TOKEN)",
        ));
    }
    token::authenticate(root, presented).map_err(|e| err(StatusCode::UNAUTHORIZED, e.to_string()))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        api: API_VERSION.into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

#[derive(Deserialize)]
struct PathQuery {
    path: String,
}

async fn get_concept(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<PathQuery>,
) -> Result<Json<ConceptResponse>, ApiError> {
    let _agent = agent_from(&headers, &st.root)?;
    let c = okf::read_concept(&st.root, &q.path)
        .map_err(|e| err(StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(ConceptResponse {
        rel: c.rel,
        r#type: c.frontmatter.r#type,
        title: c.frontmatter.title,
        description: c.frontmatter.description,
        tags: c.frontmatter.tags,
        body: c.body,
    }))
}

async fn post_claim(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ClaimRequest>,
) -> Result<Json<ClaimResponse>, ApiError> {
    let agent = agent_from(&headers, &st.root)?;
    let _guard = st.mutex.lock().await;
    let cfg = Config::load(&st.root).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rel = lock::normalize_concept(&body.concept);
    okf::read_concept(&st.root, &rel).map_err(|e| err(StatusCode::NOT_FOUND, e.to_string()))?;
    let claim = lock::try_claim(&st.root, &cfg, &rel, &agent)
        .map_err(|e| err(StatusCode::CONFLICT, e.to_string()))?;
    Ok(Json(ClaimResponse {
        concept: claim.concept,
        agent: claim.agent,
        claimed_at: claim.claimed_at.to_rfc3339(),
    }))
}

async fn post_release(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ReleaseRequest>,
) -> Result<Json<ReleaseResponse>, ApiError> {
    let agent = agent_from(&headers, &st.root)?;
    let _guard = st.mutex.lock().await;
    let cfg = Config::load(&st.root).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rel = lock::normalize_concept(&body.concept);
    let path = lock::lock_path(&st.root, &cfg, &rel);
    match lock::read_lock(&path).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        None => Err(err(StatusCode::NOT_FOUND, format!("no lock found for '{rel}'"))),
        Some(existing) => {
            if existing.agent != agent && !body.force {
                return Err(err(
                    StatusCode::FORBIDDEN,
                    format!(
                        "lock for '{rel}' is held by '{}' (you are '{agent}'). Use --force to override.",
                        existing.agent
                    ),
                ));
            }
            lock::remove_lock(&path).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            Ok(Json(ReleaseResponse {
                concept: rel,
                was_held_by: existing.agent,
            }))
        }
    }
}

async fn post_proposal(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ProposeRequest>,
) -> Result<Json<ProposeResponse>, ApiError> {
    let agent = agent_from(&headers, &st.root)?;
    let _guard = st.mutex.lock().await;
    let out = crate::propose::run(
        &st.root,
        &body.concept,
        &agent,
        &body.markdown,
        body.title.as_deref(),
        st.push,
    )
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not claimed") || msg.contains("bagsied by") {
            err(StatusCode::CONFLICT, msg)
        } else if msg.contains("invalid OKF") || msg.contains("not found") {
            err(StatusCode::BAD_REQUEST, msg)
        } else {
            err(StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;
    Ok(Json(ProposeResponse {
        concept: out.rel,
        agent,
        committed: out.committed,
        pushed: out.pushed,
        message: "ok".into(),
    }))
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LintQuery {
    #[serde(default)]
    strict: bool,
}

async fn get_lint(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(_q): Query<LintQuery>,
) -> Result<Json<LintResponse>, ApiError> {
    let _agent = agent_from(&headers, &st.root)?;
    crate::lint::collect(&st.root).map(Json).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn fallback() -> impl IntoResponse {
    err(StatusCode::NOT_FOUND, "unknown path — bagsy API is /health and /v1/…")
}

pub async fn run(root: PathBuf, bind: SocketAddr, push: bool, push_interval: u64) -> Result<()> {
    if !root.join("concepts").is_dir() && !root.join(".bagsy").is_dir() {
        anyhow::bail!(
            "{} does not look like a bagsy KB (need concepts/ or .bagsy/). Run `bagsy init`.",
            root.display()
        );
    }

    let n_active = token::list(&root)?
        .iter()
        .filter(|t| t.is_active())
        .count();
    if n_active == 0 {
        eprintln!("warning: no active agent tokens — run `bagsy token create --agent <id>`");
    }

    let state = Arc::new(AppState {
        root: root.clone(),
        mutex: Mutex::new(()),
        push,
    });

    if push_interval > 0 {
        let push_root = root.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(push_interval));
            loop {
                ticker.tick().await;
                let r = push_root.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    if git::is_git_repo(&r) && git::has_remote(&r, "origin") {
                        if let Err(e) = git::push_head(&r) {
                            eprintln!("periodic push failed: {e}");
                        }
                    }
                })
                .await;
            }
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/health", get(health))
        .route("/v1/concepts", get(get_concept))
        .route("/v1/claims", post(post_claim))
        .route("/v1/releases", post(post_release))
        .route("/v1/proposals", post(post_proposal))
        .route("/v1/lint", get(get_lint))
        .fallback(fallback)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    let local = listener.local_addr()?;
    println!("bagsy serve listening on http://{local}");
    println!("  data dir: {}", root.display());
    println!("  api:      /health  /v1/concepts|claims|releases|proposals|lint");
    if n_active == 0 {
        println!("  tokens:   none (create with bagsy token create --agent <id> --root {})", root.display());
    } else {
        println!("  tokens:   {n_active} active");
    }
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server")?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    eprintln!("bagsy serve: shutting down");
}

pub fn run_blocking(root: PathBuf, bind: SocketAddr, push: bool, push_interval: u64) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    rt.block_on(run(root, bind, push, push_interval))
}
