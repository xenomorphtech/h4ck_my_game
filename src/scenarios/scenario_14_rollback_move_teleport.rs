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
        sprite: "wall",
        x: 3,
        y: 3,
        label: "Wall",
    },
    SceneEntity {
        sprite: "relic",
        x: 6,
        y: 3,
        label: "Relic #77",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[
    BlockedTile {
        x: 3,
        y: 2,
        reason: "wall",
    },
    BlockedTile {
        x: 3,
        y: 3,
        reason: "wall",
    },
    BlockedTile {
        x: 3,
        y: 4,
        reason: "wall",
    },
];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "14-rollback-move-teleport"
    }
    fn title(&self) -> &'static str {
        "Rollback Dash: Move Intent Teleport"
    }
    fn player_title(&self) -> &'static str {
        "Ruins 1"
    }
    fn category(&self) -> &'static str {
        "Ruins"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Reach the relic behind the wall."
    }
    fn lesson(&self) -> &'static str {
        "Movement validation lagged behind client intents, letting several same-frame intents pass before wall collision was committed. Fix: validate movement against authoritative position after every step."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "MoveIntent { seq: Int, dx: Int, dy: Int }",
            "Interact { target: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  send MoveIntent { seq: 1, dx: 2, dy: 0 }\n  send MoveIntent { seq: 2, dx: 2, dy: 0 }\n  send MoveIntent { seq: 3, dx: 2, dy: 0 }\n  send MoveIntent { seq: 4, dx: 2, dy: 0 }\n  send Interact { target: 77 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send MoveIntent { seq: 1, dx: 1, dy: 0 }\nsleep 100\nsend Interact { target: 77 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "ruins",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events
            .iter()
            .filter(|x| x.name == "MoveIntent" && x.t == 0)
            .count()
            >= 4
            && events
                .iter()
                .any(|x| x.name == "Interact" && x.t == 0 && field_i64(x, "target") == Some(77))
    }
}
