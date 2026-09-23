//! Log in with EVE SSO and store the refresh token in `.env`.
//!
//! Reads the application settings from `.env` (see `.env.example`), prints
//! the login URL, and asks for the `code` query parameter from the URL the
//! browser is redirected to after login. The resulting refresh token is
//! written to `ESI_REFRESH_TOKEN` in `.env`, ready for
//! `cargo test --test authenticated -- --nocapture`.
//!
//! ```sh
//! cargo run --example auth_get_refresh_token
//! ```

#[path = "../tests/common/mod.rs"]
mod common;

use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = common::load()?;
    let mut esi = common::build_esi(&cfg)?;
    let auth = esi.get_authorize_url()?;

    println!("1. Open this URL and log in with the character to test with:\n");
    println!("{}\n", auth.authorization_url);
    println!(
        "2. After login the browser goes to {}?code=...&state=...",
        cfg.callback_url
    );
    println!("   (nothing needs to listen there; copy the address from the browser)\n");
    print!("Paste the full callback URL or just the code: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim();
    let mut code = input.to_owned();
    if let Some((_, query)) = input.split_once('?') {
        for pair in query.split('&') {
            match pair.split_once('=') {
                Some(("code", value)) => code = value.to_owned(),
                Some(("state", value)) if value != auth.state => {
                    return Err("state mismatch: the URL is not from this login".into())
                }
                _ => {}
            }
        }
    }

    let claims = esi.authenticate(&code, auth.pkce_verifier).await?;
    let refresh_token = esi
        .refresh_token
        .clone()
        .ok_or("SSO returned no refresh token")?;
    common::save_refresh_token(&refresh_token)?;

    let who = claims
        .map(|c| format!("{} ({})", c.name, c.sub))
        .or_else(|| {
            esi.access_token
                .as_deref()
                .and_then(common::character_id_from_token)
                .map(|id| id.to_string())
        })
        .unwrap_or_else(|| "unknown character".to_owned());
    println!(
        "\nLogged in as {who}; ESI_REFRESH_TOKEN saved to {}",
        common::env_path().display()
    );
    Ok(())
}
