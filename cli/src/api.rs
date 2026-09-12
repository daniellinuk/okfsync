//! Shared JSON types for the okfsync HTTP API (v1).

use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub api: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptResponse {
    pub rel: String,
    pub r#type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub body: String,
    /// Full document (frontmatter + body) for round-trip into propose.
    #[serde(default)]
    pub markdown: String,
    #[serde(default)]
    pub updated_by: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoamiResponse {
    pub ok: bool,
    pub agent: String,
    pub token_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorResponse {
    pub ok: bool,
    pub version: String,
    pub api: String,
    pub agent: String,
    pub token_id: String,
    pub concepts: usize,
    #[serde(default)]
    pub by_type: std::collections::BTreeMap<String, usize>,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub git_repo: bool,
    pub git_branch: Option<String>,
    pub git_origin: bool,
    pub tokens_active: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeRequest {
    pub concept: String,
    /// Full markdown document (frontmatter + body).
    pub markdown: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeResponse {
    pub concept: String,
    pub agent: String,
    pub committed: bool,
    pub pushed: bool,
    pub message: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintResponse {
    pub concepts: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Index row for list/search (no body — get the page next).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptSummary {
    pub rel: String,
    pub r#type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub updated_by: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesResponse {
    pub concepts: Vec<ConceptSummary>,
}
