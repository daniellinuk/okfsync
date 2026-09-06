//! Per-agent bearer tokens stored on the server data dir.
//!
//! The owner of the KB (whoever can write `--root`) creates and revokes tokens
//! locally. Agents never see `.bagsy/tokens.toml` — only the one-time secret.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const TOKENS_FILE: &str = ".bagsy/tokens.toml";
const TOKEN_PREFIX: &str = "bgy";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    pub id: String,
    pub agent: String,
    /// Hex-encoded SHA-256 of the full token string (`bgy_<id>_<secret>`).
    pub secret_hash: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
}

impl TokenRecord {
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenStore {
    #[serde(default)]
    pub tokens: Vec<TokenRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssuedToken {
    pub id: String,
    pub agent: String,
    pub token: String,
}

pub fn tokens_path(root: &Path) -> PathBuf {
    root.join(TOKENS_FILE)
}

pub fn load(root: &Path) -> Result<TokenStore> {
    let path = tokens_path(root);
    if !path.exists() {
        return Ok(TokenStore::default());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(TokenStore::default());
    }
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn save(root: &Path, store: &TokenStore) -> Result<()> {
    let path = tokens_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(store).context("serializing tokens")?;
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, text).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, &path).with_context(|| format!("renaming {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn random_hex(nbytes: usize) -> String {
    let mut buf = vec![0u8; nbytes];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

pub fn create(root: &Path, agent: &str, rotate: bool) -> Result<IssuedToken> {
    let agent = agent.trim();
    if agent.is_empty() {
        bail!("agent name must not be empty");
    }
    let mut store = load(root)?;
    let active: Vec<&TokenRecord> = store
        .tokens
        .iter()
        .filter(|t| t.agent == agent && t.is_active())
        .collect();
    if !active.is_empty() && !rotate {
        bail!(
            "agent '{agent}' already has an active token (id {}).\n  bagsy token create --agent {agent} --rotate\n  bagsy token revoke --agent {agent}",
            active
                .iter()
                .map(|t| t.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if rotate {
        let now = Utc::now();
        for t in store.tokens.iter_mut() {
            if t.agent == agent && t.is_active() {
                t.revoked_at = Some(now);
            }
        }
    }

    let id = random_hex(8);
    let secret = random_hex(32);
    let token = format!("{TOKEN_PREFIX}_{id}_{secret}");
    let record = TokenRecord {
        id: id.clone(),
        agent: agent.to_string(),
        secret_hash: hash_token(&token),
        created_at: Utc::now(),
        revoked_at: None,
    };
    store.tokens.push(record);
    save(root, &store)?;
    Ok(IssuedToken {
        id,
        agent: agent.to_string(),
        token,
    })
}

pub fn list(root: &Path) -> Result<Vec<TokenRecord>> {
    Ok(load(root)?.tokens)
}

pub fn revoke_by_id(root: &Path, id: &str, dry_run: bool) -> Result<(TokenRecord, bool)> {
    let mut store = load(root)?;
    let rec = store
        .tokens
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no token with id '{id}'\n  bagsy token list\n  bagsy token revoke --id <token-id>"
            )
        })?;
    if rec.revoked_at.is_some() {
        return Ok((rec.clone(), false));
    }
    if dry_run {
        return Ok((rec.clone(), true));
    }
    rec.revoked_at = Some(Utc::now());
    let cloned = rec.clone();
    save(root, &store)?;
    Ok((cloned, true))
}

pub fn revoke_by_agent(
    root: &Path,
    agent: &str,
    dry_run: bool,
) -> Result<(Vec<TokenRecord>, bool)> {
    let mut store = load(root)?;
    let now = Utc::now();
    let mut revoked = Vec::new();
    for t in store.tokens.iter_mut() {
        if t.agent == agent && t.is_active() {
            if !dry_run {
                t.revoked_at = Some(now);
            }
            revoked.push(t.clone());
        }
    }
    if revoked.is_empty() {
        let known = store.tokens.iter().any(|t| t.agent == agent);
        if known {
            return Ok((vec![], false));
        }
        bail!(
            "no token for agent '{agent}'\n  bagsy token list\n  bagsy token create --agent {agent}"
        );
    }
    if !dry_run {
        save(root, &store)?;
    }
    Ok((revoked, true))
}

/// Resolve a presented bearer token to the agent name, if it is active.
pub fn authenticate(root: &Path, presented: &str) -> Result<String> {
    let presented = presented.trim();
    if presented.is_empty() {
        bail!("missing token");
    }
    let hash = hash_token(presented);
    let store = load(root)?;
    match store.tokens.iter().find(|t| t.secret_hash == hash) {
        None => bail!("invalid token"),
        Some(t) if !t.is_active() => bail!("token '{}' has been revoked", t.id),
        Some(t) => Ok(t.agent.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn root() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".bagsy")).unwrap();
        tmp
    }

    #[test]
    fn create_authenticate_revoke() {
        let tmp = root();
        let issued = create(tmp.path(), "alice", false).unwrap();
        assert_eq!(authenticate(tmp.path(), &issued.token).unwrap(), "alice");
        revoke_by_id(tmp.path(), &issued.id, false).unwrap();
        assert!(authenticate(tmp.path(), &issued.token).is_err());
        let (_, changed) = revoke_by_id(tmp.path(), &issued.id, false).unwrap();
        assert!(!changed);
    }

    #[test]
    fn one_active_token_per_agent_unless_rotate() {
        let tmp = root();
        create(tmp.path(), "bob", false).unwrap();
        assert!(create(tmp.path(), "bob", false).is_err());
        let rotated = create(tmp.path(), "bob", true).unwrap();
        assert_eq!(authenticate(tmp.path(), &rotated.token).unwrap(), "bob");
    }

    #[test]
    fn revoke_agent_kills_all_active() {
        let tmp = root();
        create(tmp.path(), "carol", false).unwrap();
        let (n, changed) = revoke_by_agent(tmp.path(), "carol", false).unwrap();
        assert_eq!(n.len(), 1);
        assert!(changed);
        let (n2, changed2) = revoke_by_agent(tmp.path(), "carol", false).unwrap();
        assert!(n2.is_empty());
        assert!(!changed2);
    }
}
