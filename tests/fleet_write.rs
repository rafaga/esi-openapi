//! Live test for the fleet endpoints that change a fleet.
//!
//! It is ignored by default because it modifies the fleet the authenticated
//! character is in. To run it, log in as the fleet boss, make sure the
//! refresh token has the `esi-fleets.read_fleet.v1` and
//! `esi-fleets.write_fleet.v1` scopes, and run:
//!
//! ```sh
//! cargo test --test fleet_write -- --ignored --nocapture
//! ```
//!
//! The test creates a wing and a squad, renames both, and deletes them
//! again. The wing is deleted even when an earlier step fails.

mod common;

use esi_openapi::groups::{FleetNaming, FleetRole};

#[tokio::test]
#[ignore = "modifies the fleet the character is in; run it explicitly"]
async fn fleet_wing_and_squad_lifecycle() {
    let cfg = match common::load() {
        Ok(cfg) => cfg,
        Err(reason) => {
            eprintln!("skipping fleet write test: {reason}");
            return;
        }
    };
    let (esi, character_id) = match common::authenticated_esi(&cfg).await {
        Ok(v) => v,
        Err(reason) if reason.starts_with("missing") => {
            eprintln!("skipping fleet write test: {reason}");
            return;
        }
        Err(reason) => panic!("{reason}"),
    };

    let fleet = match esi.group_fleets().get_character_fleet(character_id).await {
        Ok(fleet) => fleet,
        Err(e) => {
            eprintln!(
                "skipping fleet write test: character {character_id} is not in a fleet ({e})"
            );
            return;
        }
    };
    if fleet.role != FleetRole::FleetCommander {
        eprintln!(
            "skipping fleet write test: character {character_id} is {:?}, not the fleet commander",
            fleet.role
        );
        return;
    }
    let fleet_id = fleet.fleet_id;
    let groups = esi.group_fleets();

    let wing = groups
        .create_wing(fleet_id)
        .await
        .expect("create_wing failed");
    println!("created wing {}", wing.wing_id);

    let steps = async {
        let name = FleetNaming {
            name: "esi-openapi test wing".to_owned(),
        };
        groups
            .rename_wing(fleet_id, wing.wing_id, &name)
            .await
            .map_err(|e| format!("rename_wing: {e}"))?;

        let squad = groups
            .create_squad(fleet_id, wing.wing_id)
            .await
            .map_err(|e| format!("create_squad: {e}"))?;
        println!("created squad {}", squad.squad_id);

        let name = FleetNaming {
            name: "esi-openapi test squad".to_owned(),
        };
        groups
            .rename_squad(fleet_id, squad.squad_id, &name)
            .await
            .map_err(|e| format!("rename_squad: {e}"))?;

        let wings = groups
            .get_wings(fleet_id)
            .await
            .map_err(|e| format!("get_wings: {e}"))?;
        let found = wings
            .iter()
            .find(|w| w.id == wing.wing_id)
            .ok_or_else(|| "the new wing is not listed".to_owned())?;
        if !found.squads.iter().any(|s| s.id == squad.squad_id) {
            return Err("the new squad is not listed".to_owned());
        }

        groups
            .delete_squad(fleet_id, squad.squad_id)
            .await
            .map_err(|e| format!("delete_squad: {e}"))?;
        Ok::<(), String>(())
    }
    .await;

    // Always try to remove the wing, which also removes any squad left in it.
    let cleanup = groups.delete_wing(fleet_id, wing.wing_id).await;
    if let Err(e) = &cleanup {
        eprintln!("could not delete wing {}: {e}", wing.wing_id);
    }
    steps.expect("fleet write steps failed");
    cleanup.expect("delete_wing failed");
}
