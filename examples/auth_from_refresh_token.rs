//! Get an access token from the refresh token in `.env` and call an
//! authenticated endpoint.
//!
//! ```sh
//! cargo run --example auth_from_refresh_token
//! ```

#[path = "../tests/common/mod.rs"]
mod common;

use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let cfg = common::load()?;
    let (esi, character_id) = common::authenticated_esi(&cfg).await?;
    let skills = esi.group_skills().get_skills(character_id).await?;
    info!(
        "Character {character_id} has {} SP in {} skills",
        skills.total_sp,
        skills.skills.len()
    );

    Ok(())
}
