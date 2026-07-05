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
        sprite: "monster",
        x: 4,
        y: 2,
        label: "Monster #1 (120 HP)",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "01-first-blood-batch"
    }
    fn title(&self) -> &'static str {
        "First Blood: Batch the Fatal Window"
    }
    fn player_title(&self) -> &'static str {
        "Arena 1"
    }
    fn category(&self) -> &'static str {
        "Arena"
    }
    fn difficulty(&self) -> &'static str {
        "★☆☆"
    }
    fn objective(&self) -> &'static str {
        "Kill the monster."
    }
    fn lesson(&self) -> &'static str {
        "The server resolved damage per received packet and let the monster retaliate between your hits. Delivering three attacks inside one tick collapsed the retaliation window. Fix: resolve an action and its reactions atomically on the server, not once per inbound packet."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Attack { target: Int, power: Int = 40 }"]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 100\nsend Attack { target: 1 }\nsleep 100\nsend Attack { target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "arena",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        same_tick_count(events, "Attack", "target", 1, 3)
    }
}
