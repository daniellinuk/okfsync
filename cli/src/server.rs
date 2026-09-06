//! `bagsy serve` — HTTP API over one OKF data directory.

use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, Method, StatusCode};
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
    ConceptResponse, ErrorBody, HealthResponse, LintResponse, PagesResponse, ProposeRequest,
    ProposeResponse, API_VERSION,
};
use crate::git;
use crate::okf;
use crate::token;

struct AppState {
    root: PathBuf,
    mutex: Mutex<()>,
    push: bool,
}

type ApiError = (StatusCode, Json<ErrorBody>);

fn err(status: StatusCode, msg: impl Into<String>) -> ApiError {
    (status, Json(ErrorBody { error: msg.into() }))
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
            return token::authenticate(root, alt.trim())
                .map_err(|e| err(StatusCode::UNAUTHORIZED, e.to_string()));
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
    let doc = okf::load_document(&st.root, &q.path)
        .map_err(|e| err(StatusCode::NOT_FOUND, e.to_string()))?;
    let c = doc.concept;
    Ok(Json(ConceptResponse {
        rel: c.rel,
        r#type: c.frontmatter.r#type,
        title: c.frontmatter.title,
        description: c.frontmatter.description,
        tags: c.frontmatter.tags,
        body: c.body,
        markdown: doc.markdown,
    }))
}

#[derive(Deserialize)]
struct PagesQuery {
    #[serde(default)]
    q: Option<String>,
}

async fn list_pages(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<PagesQuery>,
) -> Result<Json<PagesResponse>, ApiError> {
    let _agent = agent_from(&headers, &st.root)?;
    let concepts = match q.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(query) => crate::okf::search_concepts(&st.root, query)
            .map_err(|e| err(StatusCode::BAD_REQUEST, e.to_string()))?,
        None => crate::okf::summaries(&st.root)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };
    Ok(Json(PagesResponse { concepts }))
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
        false,
    )
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("invalid") || msg.contains("refusing") || msg.contains("must be") {
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
    crate::lint::collect(&st.root)
        .map(Json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn reject_delete() -> impl IntoResponse {
    err(
        StatusCode::METHOD_NOT_ALLOWED,
        "bagsy CLI/API cannot delete knowledge; gardening is out of band (git history)",
    )
}

async fn fallback(method: Method) -> impl IntoResponse {
    if method == Method::DELETE {
        return reject_delete().await.into_response();
    }
    err(
        StatusCode::NOT_FOUND,
        "unknown path — bagsy API is /health and /v1/…",
    )
    .into_response()
}

pub async fn run(
    root: PathBuf,
    bind: SocketAddr,
    push: bool,
    push_interval: u64,
    json: bool,
) -> Result<()> {
    if !root.join("concepts").is_dir() && !root.join(".bagsy").is_dir() {
        anyhow::bail!(
            "{} does not look like a bagsy KB (need concepts/ or .bagsy/). Run `bagsy init`.",
            root.display()
        );
    }
    crate::config::Config::load(&root)?;

    let n_active = token::list(&root)?.iter().filter(|t| t.is_active()).count();
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
        .route("/health", get(health).delete(reject_delete))
        .route("/v1/health", get(health).delete(reject_delete))
        .route("/v1/concepts", get(get_concept).delete(reject_delete))
        .route("/v1/pages", get(list_pages).delete(reject_delete))
        .route("/v1/proposals", post(post_proposal).delete(reject_delete))
        .route("/v1/lint", get(get_lint).delete(reject_delete))
        .fallback(fallback)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    let local = listener.local_addr()?;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "url": format!("http://{local}"),
                "root": root.display().to_string(),
                "api": API_VERSION,
                "tokens": n_active,
            })
        );
    } else {
        println!("bagsy serve listening on http://{local}");
        println!("  data dir: {}", root.display());
        println!("  api:      /health  /v1/concepts|pages|proposals|lint  (no delete)");
        if n_active == 0 {
            println!(
                "  tokens:   none (create with bagsy token create --agent <id> --root {})",
                root.display()
            );
        } else {
            println!("  tokens:   {n_active} active");
        }
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

pub fn run_blocking(
    root: PathBuf,
    bind: SocketAddr,
    push: bool,
    push_interval: u64,
    json: bool,
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    rt.block_on(run(root, bind, push, push_interval, json))
}
