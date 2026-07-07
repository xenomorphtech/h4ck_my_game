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
        x: 0,
        y: 3,
        label: "You",
    },
    SceneEntity {
        sprite: "guard",
        x: 3,
        y: 3,
        label: "Guard",
    },
    SceneEntity {
        sprite: "gate",
        x: 6,
        y: 3,
        label: "Gate",
    },
    SceneEntity {
        sprite: "key",
        x: 6,
        y: 2,
        label: "Lever #7",
    },
];
// The approach is a single corridor on row y=3: walls fence the guard's tile so
// there is no way around, only through.
const BLOCKED_TILES: &[BlockedTile] = &[
    BlockedTile {
        x: 3,
        y: 2,
        reason: "wall",
    },
    BlockedTile {
        x: 3,
        y: 3,
        reason: "guard patrol",
    },
    BlockedTile {
        x: 3,
        y: 4,
        reason: "wall",
    },
    BlockedTile {
        x: 6,
        y: 3,
        reason: "locked gate",
    },
];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "02-target-validation-range"
    }
    fn title(&self) -> &'static str {
        "Long Arm: Range Validation Bypass"
    }
    fn player_title(&self) -> &'static str {
        "Gatehouse 1"
    }
    fn category(&self) -> &'static str {
        "Gatehouse"
    }
    fn difficulty(&self) -> &'static str {
        "★☆☆"
    }
    fn objective(&self) -> &'static str {
        "Open the gate with its lever. A guard holds the only corridor to it."
    }
    fn lesson(&self) -> &'static str {
        "The Use packet reached the lever directly; the route and range check only existed in the normal client flow."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Move { x: Int, y: Int }", "Use { target: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send Use { target: 7 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "for x in 1..10 {\n  send Move { x: x, y: 0 }\n  sleep 50\n}\nsend Use { target: 7 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "gatehouse",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        has(events, "Use", "target", 7) && !events.iter().any(|x| x.name == "Move")
    }
}
