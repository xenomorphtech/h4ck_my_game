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
        sprite: "bridge",
        x: 3,
        y: 3,
        label: "Broken Bridge",
    },
    SceneEntity {
        sprite: "shrine",
        x: 7,
        y: 3,
        label: "Shrine #91",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[
    BlockedTile {
        x: 3,
        y: 3,
        reason: "chasm",
    },
    BlockedTile {
        x: 4,
        y: 3,
        reason: "chasm",
    },
    BlockedTile {
        x: 5,
        y: 3,
        reason: "chasm",
    },
];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "21-telehacking-position-spoof"
    }
    fn title(&self) -> &'static str {
        "Blink Without Moving: Telehacking Position Spoof"
    }
    fn player_title(&self) -> &'static str {
        "Bridge 1"
    }
    fn category(&self) -> &'static str {
        "Bridge"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Activate the far shrine across the chasm."
    }
    fn lesson(&self) -> &'static str {
        "The interaction packet carried its own position, letting you activate the shrine without actually crossing."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "Move { x: Int, y: Int }",
            "Interact { target: Int, client_x: Int, client_y: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send Interact { target: 91, client_x: 7, client_y: 3 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "for x in 1..8 {\n  send Move { x: x, y: 3 }\n  sleep 20\n}\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "bridge",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events.iter().any(|x| {
            x.name == "Interact"
                && field_i64(x, "target") == Some(91)
                && field_i64(x, "client_x") == Some(7)
                && field_i64(x, "client_y") == Some(3)
        }) && !events.iter().any(|x| x.name == "Move")
    }
}
