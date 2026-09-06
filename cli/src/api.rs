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
    #[serde(default)]
    pub body: String,
    /// Full document (frontmatter + body) for round-trip into propose.
    #[serde(default)]
    pub markdown: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesResponse {
    pub concepts: Vec<ConceptSummary>,
}
