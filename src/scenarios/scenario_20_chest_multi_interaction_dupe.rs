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
        label: "Chest #81",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "20-chest-multi-interaction-dupe"
    }
    fn title(&self) -> &'static str {
        "Many Hands: Chest Multi-Interaction Dupe"
    }
    fn player_title(&self) -> &'static str {
        "Vault 2"
    }
    fn category(&self) -> &'static str {
        "Vault"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Collect three reward bundles."
    }
    fn lesson(&self) -> &'static str {
        "Opening dispensed rewards before the chest was flagged empty, so simultaneous opens each saw it as full. Fix: flip the chest to looted atomically with the first successful open."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["OpenChest { chest: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  send OpenChest { chest: 81 }\n  send OpenChest { chest: 81 }\n  send OpenChest { chest: 81 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send OpenChest { chest: 81 }\nsleep 50\nsend OpenChest { chest: 81 }\nsleep 50\nsend OpenChest { chest: 81 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "vault",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        same_tick_count(events, "OpenChest", "chest", 81, 3)
    }
}
