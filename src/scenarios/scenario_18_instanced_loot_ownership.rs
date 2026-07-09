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
        sprite: "boss",
        x: 4,
        y: 2,
        label: "Raid Boss #1",
    },
    SceneEntity {
        sprite: "ally",
        x: 6,
        y: 2,
        label: "Ally with drop #7002",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "18-instanced-loot-ownership"
    }
    fn title(&self) -> &'static str {
        "Not Your Sparkle: Loot Ownership Bypass"
    }
    fn player_title(&self) -> &'static str {
        "Raid 1"
    }
    fn category(&self) -> &'static str {
        "Raid"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Take another party member's drop."
    }
    fn lesson(&self) -> &'static str {
        "The loot claim path accepted a drop you could see, even though it belonged to another player."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Attack { target: Int }", "Loot { drop: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsend Loot { drop: 7002 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\n"
    }
    fn upcoming(&self) -> bool {
        true
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "raid",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        has(events, "Attack", "target", 1) && has(events, "Loot", "drop", 7002)
    }
}
