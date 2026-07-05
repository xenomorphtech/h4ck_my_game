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
        label: "Arena Monster #1 (160 HP)",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "02-arena-fight-while-dead"
    }
    fn title(&self) -> &'static str {
        "Second Wind: Dead Player Action Accepted"
    }
    fn player_title(&self) -> &'static str {
        "Arena 2"
    }
    fn category(&self) -> &'static str {
        "Arena"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Kill the monster."
    }
    fn lesson(&self) -> &'static str {
        "The combat handler checked target health but never checked that the attacker was alive. A fatal counterattack should move the player into a state where normal actions are rejected unless the scenario explicitly allows ghost actions. Fix: validate actor liveness on every action path, not just movement/UI input."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Attack { target: Int, power: Int = 40 }"]
    }
    fn solution_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 300\nbatch {\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 300\nsend Attack { target: 1 }\nsleep 100\nsend Attack { target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "arena",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events
            .iter()
            .any(|x| x.name == "Attack" && x.t < 250 && field_i64(x, "target") == Some(1))
            && events.iter().any(|anchor| {
                anchor.name == "Attack"
                    && anchor.t >= 250
                    && field_i64(anchor, "target") == Some(1)
                    && events
                        .iter()
                        .filter(|x| {
                            x.t == anchor.t
                                && x.name == "Attack"
                                && field_i64(x, "target") == Some(1)
                        })
                        .count()
                        >= 3
            })
    }
}
