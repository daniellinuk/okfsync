//! HTTP client for an okfsync server.

use anyhow::{bail, Context, Result};
use std::path::Path;
use ureq::Error as UreqError;

use crate::api::{
    ConceptResponse, DoctorResponse, ErrorBody, HealthResponse, LintResponse, PagesResponse,
    ProposeRequest, ProposeResponse, WhoamiResponse,
};
use crate::token;

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

    pub fn list_pages(&self) -> Result<PagesResponse> {
        read_json(
            self.auth(ureq::get(&format!("{}/v1/pages", self.url)))
                .call(),
        )
    }

    pub fn search_pages(&self, query: &str) -> Result<PagesResponse> {
        read_json(
            self.auth(ureq::get(&format!("{}/v1/pages", self.url)))
                .query("q", query)
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

    pub fn whoami(&self) -> Result<WhoamiResponse> {
        read_json(
            self.auth(ureq::get(&format!("{}/v1/whoami", self.url)))
                .call(),
        )
    }

    pub fn doctor(&self) -> Result<DoctorResponse> {
        read_json(
            self.auth(ureq::get(&format!("{}/v1/doctor", self.url)))
                .call(),
        )
    }
}

/// Unauthenticated reachability check (short timeout).
pub fn probe_health(url: &str) -> Result<HealthResponse> {
    let base = url.trim().trim_end_matches('/');
    read_json(
        ureq::get(&format!("{base}/health"))
            .timeout(std::time::Duration::from_secs(3))
            .call(),
    )
}

/// Client mode when URL + a bearer are present (--token, KBSYNC_TOKEN, or token file).
pub fn from_opts(
    url: Option<&str>,
    token: Option<&str>,
    token_file: Option<&Path>,
) -> Result<Option<Remote>> {
    let url = nonempty(url).or_else(|| nonempty_env("KBSYNC_URL"));
    let token = resolve_token(token, token_file)?;
    match (url, token) {
        (None, None) => Ok(None),
        (Some(_), None) => bail!(
            "KBSYNC_TOKEN is missing (KBSYNC_URL is set)\n  kbsync get <concept> --url http://127.0.0.1:7432 --token <token>\n  export KBSYNC_TOKEN=kbs_…\n  export KBSYNC_TOKEN_FILE=/path/to/agent.token"
        ),
        (None, Some(_)) => bail!(
            "KBSYNC_URL is missing (token is set)\n  kbsync get <concept> --url http://127.0.0.1:7432 --token <token>\n  export KBSYNC_URL=http://127.0.0.1:7432"
        ),
        (Some(url), Some(token)) => Ok(Some(Remote::new(&url, &token))),
    }
}

fn resolve_token(flag: Option<&str>, token_file: Option<&Path>) -> Result<Option<String>> {
    let file_path = token_file
        .map(|p| p.to_path_buf())
        .or_else(|| nonempty_env("KBSYNC_TOKEN_FILE").map(std::path::PathBuf::from));

    if let Some(t) = nonempty(flag).or_else(|| nonempty_env("KBSYNC_TOKEN")) {
        token::validate_bearer_shape(&t)?;
        if file_path.is_some() {
            eprintln!("kbsync: token is set via --token/KBSYNC_TOKEN; ignoring KBSYNC_TOKEN_FILE");
        }
        return Ok(Some(t));
    }

    if let Some(path) = file_path {
        return Ok(Some(load_token_file(&path)?));
    }
    Ok(None)
}

fn load_token_file(path: &Path) -> Result<String> {
    let text = std::fs::read_to_string(path).with_context(|| {
        format!(
            "reading token file {}\n  file should contain the kbs_… secret (chmod 600)\n  export KBSYNC_TOKEN_FILE={}",
            path.display(),
            path.display()
        )
    })?;
    warn_token_file_perms(path);
    let token = extract_token_file_contents(&text)?;
    token::validate_bearer_shape(&token)?;
    Ok(token)
}

fn extract_token_file_contents(text: &str) -> Result<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        bail!(
            "token file is empty\n  put the kbs_… secret in the file (chmod 600)\n  export KBSYNC_TOKEN_FILE=/path/to/agent.token"
        );
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        token::validate_bearer_shape(trimmed)?;
    }
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let rest = line
            .strip_prefix("export KBSYNC_TOKEN=")
            .or_else(|| line.strip_prefix("KBSYNC_TOKEN="));
        if let Some(rest) = rest {
            return Ok(unquote(rest).to_string());
        }
    }
    if trimmed.contains('\n') {
        bail!(
            "token file has multiple lines and no KBSYNC_TOKEN= line\n  use a file that contains only the kbs_… secret\n  or KBSYNC_TOKEN=kbs_…"
        );
    }
    Ok(unquote(trimmed).to_string())
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        if (bytes[0] == b'"' && bytes[s.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[s.len() - 1] == b'\'')
        {
            return &s[1..s.len() - 1];
        }
    }
    s
}

fn warn_token_file_perms(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode() & 0o777;
            if mode & 0o077 != 0 {
                eprintln!(
                    "kbsync: warning: {} is readable by group/other (chmod 600)",
                    path.display()
                );
            }
        }
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
        Ok(resp) => resp.into_json().context("decoding okfsync server JSON"),
        Err(UreqError::Status(_code, resp)) => {
            let msg = resp
                .into_json::<ErrorBody>()
                .map(|b| b.error)
                .unwrap_or_else(|_| "okfsync server error".into());
            bail!("{msg}")
        }
        Err(UreqError::Transport(t)) => bail!("cannot reach okfsync server: {t}"),
    }
}
