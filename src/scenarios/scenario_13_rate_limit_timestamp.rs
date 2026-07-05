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
        sprite: "crystal",
        x: 5,
        y: 2,
        label: "Crystal",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "13-rate-limit-timestamp"
    }
    fn title(&self) -> &'static str {
        "Same Time: Timestamp Rate-Limit Bypass"
    }
    fn player_title(&self) -> &'static str {
        "Crystal 1"
    }
    fn category(&self) -> &'static str {
        "Crystal"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Overload the crystal."
    }
    fn lesson(&self) -> &'static str {
        "The rate limiter bucketed by client timestamp and failed to dedupe identical timestamps. Fix: rate-limit on server arrival/order, not attacker-controlled client time."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Zap { target: Int, client_time_ms: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  for i in 1..11 {\n    send Zap { target: 1, client_time_ms: 42 }\n  }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "for i in 1..11 {\n  send Zap { target: 1, client_time_ms: i }\n  sleep 50\n}\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "cavern",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        let mut timestamps: Vec<i64> = events
            .iter()
            .filter(|x| x.name == "Zap" && field_i64(x, "target") == Some(1))
            .filter_map(|x| field_i64(x, "client_time_ms"))
            .collect();
        timestamps.sort_unstable();
        timestamps
            .windows(10)
            .any(|window| window.first() == window.last())
    }
}
