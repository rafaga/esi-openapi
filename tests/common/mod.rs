//! Shared helpers for the authenticated examples and integration tests.
//!
//! Credentials are read from a `.env` file at the crate root (see
//! `.env.example`). That file is listed in `.gitignore` and must never
//! be committed.

#![allow(dead_code)]

use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use esi_openapi::prelude::*;
use std::path::PathBuf;

/// Values loaded from `.env` (or from the process environment).
#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub callback_url: String,
    pub scopes: String,
    pub refresh_token: Option<String>,
    pub user_agent: String,
    pub structure_id: Option<i64>,
}

/// Path of the `.env` file at the crate root.
pub fn env_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env")
}

fn var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// Load `.env` and read the credentials.
///
/// Returns an error naming the missing variables when `ESI_CLIENT_ID`
/// or `ESI_CALLBACK_URL` are not set.
pub fn load() -> Result<EnvConfig, String> {
    // A missing .env is fine: the variables may come from the environment.
    match dotenvy::from_path(env_path()) {
        Ok(()) => {}
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("could not read {}: {e}", env_path().display())),
    }
    let mut missing = Vec::new();
    let client_id = var("ESI_CLIENT_ID");
    let callback_url = var("ESI_CALLBACK_URL");
    if client_id.is_none() {
        missing.push("ESI_CLIENT_ID");
    }
    if callback_url.is_none() {
        missing.push("ESI_CALLBACK_URL");
    }
    if !missing.is_empty() {
        return Err(format!(
            "missing {} (copy .env.example to .env and fill it in)",
            missing.join(", ")
        ));
    }
    let contact = var("ESI_USER_AGENT_CONTACT").unwrap_or_else(|| "unknown".to_owned());
    Ok(EnvConfig {
        client_id: client_id.unwrap(),
        client_secret: var("ESI_CLIENT_SECRET"),
        callback_url: callback_url.unwrap(),
        scopes: var("ESI_SCOPES").unwrap_or_default(),
        refresh_token: var("ESI_REFRESH_TOKEN"),
        user_agent: format!(
            "esi-openapi/{} ({contact}; +https://github.com/rafaga/esi-openapi)",
            env!("CARGO_PKG_VERSION")
        ),
        structure_id: var("ESI_TEST_STRUCTURE_ID").and_then(|v| v.parse().ok()),
    })
}

/// Build an `Esi` client from the loaded configuration. Uses the PKCE
/// (application) flow when no client secret is set.
pub fn build_esi(cfg: &EnvConfig) -> EsiResult<Esi> {
    let mut builder = EsiBuilder::new()
        .user_agent(&cfg.user_agent)
        .client_id(&cfg.client_id)
        .callback_url(&cfg.callback_url)
        .scope(&cfg.scopes);
    builder = match &cfg.client_secret {
        Some(secret) => builder.client_secret(secret),
        None => builder.enable_application_authentication(true),
    };
    builder.build()
}

/// Store a refresh token in `.env`, replacing the `ESI_REFRESH_TOKEN`
/// line (or appending one). Other lines are kept as they are.
pub fn save_refresh_token(token: &str) -> std::io::Result<()> {
    let path = env_path();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut found = false;
    let mut lines: Vec<String> = content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("ESI_REFRESH_TOKEN=") {
                found = true;
                format!("ESI_REFRESH_TOKEN={token}")
            } else {
                line.to_owned()
            }
        })
        .collect();
    if !found {
        lines.push(format!("ESI_REFRESH_TOKEN={token}"));
    }
    std::fs::write(&path, lines.join(newline) + newline)
}

/// Read the character ID from an access token's `sub` claim
/// (`CHARACTER:EVE:<id>`), without validating the token.
pub fn character_id_from_token(access_token: &str) -> Option<i32> {
    let payload = access_token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    claims["sub"].as_str()?.rsplit(':').next()?.parse().ok()
}

/// Exchange the refresh token from `.env` for an access token, saving
/// the rotated refresh token back to `.env` when EVE SSO issues a new one.
/// Returns the client and the character ID.
pub async fn authenticated_esi(cfg: &EnvConfig) -> Result<(Esi, i32), String> {
    let refresh_token = cfg
        .refresh_token
        .as_deref()
        .ok_or("missing ESI_REFRESH_TOKEN (run `cargo run --example auth_get_refresh_token`)")?;
    let mut esi = build_esi(cfg).map_err(|e| e.to_string())?;
    esi.use_refresh_token(refresh_token)
        .await
        .map_err(|e| format!("refreshing the access token failed: {e} ({e:?})"))?;
    if let Some(new_token) = esi.refresh_token.as_deref() {
        if new_token != refresh_token {
            save_refresh_token(new_token).map_err(|e| e.to_string())?;
        }
    }
    let character_id = esi
        .access_token
        .as_deref()
        .and_then(character_id_from_token)
        .ok_or("could not read the character ID from the access token")?;
    esi.update_spec().await.map_err(|e| e.to_string())?;
    Ok((esi, character_id))
}

#[cfg(test)]
mod tests {
    use super::character_id_from_token;
    use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};

    #[test]
    fn test_character_id_from_token() {
        let payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"CHARACTER:EVE:2112625428"}"#);
        let token = format!("header.{payload}.signature");
        assert_eq!(character_id_from_token(&token), Some(2112625428));
        assert_eq!(character_id_from_token("not-a-jwt"), None);
    }
}
