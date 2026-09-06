//! Shared JSON types for the bagsy HTTP API (v1).

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
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub concept: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResponse {
    pub concept: String,
    pub agent: String,
    pub claimed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseRequest {
    pub concept: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResponse {
    pub concept: String,
    pub was_held_by: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintResponse {
    pub concepts: usize,
    pub locks: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
