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
        y: 4,
        label: "You",
    },
    SceneEntity {
        sprite: "cannon",
        x: 3,
        y: 4,
        label: "Allied Cannon #2: action fire",
    },
    SceneEntity {
        sprite: "boss",
        x: 6,
        y: 2,
        label: "Commander #1 (shielded)",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "04-target-validation-faction"
    }
    fn title(&self) -> &'static str {
        "Friendly Fire: Missing Faction Gate"
    }
    fn player_title(&self) -> &'static str {
        "Siege 1"
    }
    fn category(&self) -> &'static str {
        "Siege"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Stop the shielded enemy commander."
    }
    fn lesson(&self) -> &'static str {
        "Direct attacks were faction-checked, but ordering a neutral siege unit to fire was not. The command path skipped the faction gate the attack path enforced. Fix: apply the same authorization to every code path that can deal damage."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "Attack { target: Int }",
            "OrderUnit { unit: Int, action: String, target: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send OrderUnit { unit: 2, action: \"fire\", target: 1 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "siege",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events.iter().any(|x| {
            x.name == "OrderUnit"
                && field_i64(x, "unit") == Some(2)
                && field_i64(x, "target") == Some(1)
                && field_str(x, "action") == Some("fire")
        })
    }
}
