#[allow(unused_imports)]
use super::rules::{has, same_tick_count};
use super::{BlockedTile, Scenario, Scene, SceneEntity};
#[allow(unused_imports)]
use crate::engine::{field_i64, field_str, ClientEvent};

pub struct ScenarioImpl;

pub static SCENARIO: ScenarioImpl = ScenarioImpl;

const ENTITIES: &[SceneEntity] = &[
    SceneEntity {
        sprite: "hero",
        x: 1,
        y: 3,
        label: "You",
    },
    SceneEntity {
        sprite: "chest",
        x: 4,
        y: 2,
        label: "Chest #51: grants loot #1",
    },
    SceneEntity {
        sprite: "chest",
        x: 6,
        y: 2,
        label: "Relic display",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "15-replay-signed-loot"
    }
    fn title(&self) -> &'static str {
        "Signed Once: Loot Replay"
    }
    fn player_title(&self) -> &'static str {
        "Vault 1"
    }
    fn category(&self) -> &'static str {
        "Vault"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Claim three relics from the chest."
    }
    fn lesson(&self) -> &'static str {
        "The same signed loot grant was accepted more than once, turning one authorization into multiple claims."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "OpenChest { chest: Int }",
            "ClaimLoot { chest: Int, grant_id: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send OpenChest { chest: 51 }\nfor i in 1..4 {\n  send ClaimLoot { chest: 51, grant_id: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send OpenChest { chest: 51 }\nsend ClaimLoot { chest: 51, grant_id: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "vault",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        has(events, "OpenChest", "chest", 51)
            && events
                .iter()
                .filter(|x| {
                    x.name == "ClaimLoot"
                        && field_i64(x, "chest") == Some(51)
                        && field_i64(x, "grant_id") == Some(1)
                })
                .count()
                >= 3
    }
}
