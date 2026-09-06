//! HTTP client for a bagsy server.

use anyhow::{bail, Context, Result};
use ureq::Error as UreqError;

use crate::api::{ConceptResponse, ErrorBody, LintResponse, ProposeRequest, ProposeResponse};

#[derive(Debug, Clone)]
pub struct Remote {
    pub url: String,
    pub token: String,
}

impl Remote {
    pub fn new(url: &str, token: &str) -> Self {
        Self {
            url: url.trim().trim_end_matches('/').to_string(),
            token: token.trim().to_string(),
        }
    }

    fn auth(&self, req: ureq::Request) -> ureq::Request {
        req.set("Authorization", &format!("Bearer {}", self.token))
    }

    pub fn get_concept(&self, concept: &str) -> Result<ConceptResponse> {
        read_json(
            self.auth(ureq::get(&format!("{}/v1/concepts", self.url)))
                .query("path", concept)
                .call(),
        )
    }

    pub fn propose(
        &self,
        concept: &str,
        markdown: &str,
        title: Option<&str>,
    ) -> Result<ProposeResponse> {
        read_json(
            self.auth(ureq::post(&format!("{}/v1/proposals", self.url)))
                .send_json(ProposeRequest {
                    concept: concept.to_string(),
                    markdown: markdown.to_string(),
                    title: title.map(|t| t.to_string()),
                }),
        )
    }

    pub fn lint(&self, strict: bool) -> Result<LintResponse> {
        read_json(
            self.auth(ureq::get(&format!("{}/v1/lint", self.url)))
                .query("strict", if strict { "true" } else { "false" })
                .call(),
        )
    }
}

/// Client mode when both URL and token are present.
pub fn from_opts(url: Option<&str>, token: Option<&str>) -> Result<Option<Remote>> {
    let url = nonempty(url).or_else(|| nonempty_env("BAGSY_URL"));
    let token = nonempty(token).or_else(|| nonempty_env("BAGSY_TOKEN"));
    match (url, token) {
        (None, None) => Ok(None),
        (Some(_), None) => bail!(
            "BAGSY_TOKEN is missing (BAGSY_URL is set)\n  bagsy get <concept> --url http://127.0.0.1:7432 --token <token>\n  export BAGSY_TOKEN=<token>"
        ),
        (None, Some(_)) => bail!(
            "BAGSY_URL is missing (BAGSY_TOKEN is set)\n  bagsy get <concept> --url http://127.0.0.1:7432 --token <token>\n  export BAGSY_URL=http://127.0.0.1:7432"
        ),
        (Some(url), Some(token)) => Ok(Some(Remote::new(&url, &token))),
    }
}

fn nonempty(v: Option<&str>) -> Option<String> {
    v.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn nonempty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_json<T: serde::de::DeserializeOwned>(
    result: std::result::Result<ureq::Response, UreqError>,
) -> Result<T> {
    match result {
        Ok(resp) => resp.into_json().context("decoding bagsy server JSON"),
        Err(UreqError::Status(_code, resp)) => {
            let msg = resp
                .into_json::<ErrorBody>()
                .map(|b| b.error)
                .unwrap_or_else(|_| "bagsy server error".into());
            bail!("{msg}")
        }
        Err(UreqError::Transport(t)) => bail!("cannot reach bagsy server: {t}"),
    }
}
