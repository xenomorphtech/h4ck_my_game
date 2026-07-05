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
        label: "Necromancer #1 (shielded)",
    },
    SceneEntity {
        sprite: "monster",
        x: 5,
        y: 4,
        label: "Summon #2 (shield source)",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "03-target-validation-dead"
    }
    fn title(&self) -> &'static str {
        "Ghost Target: Stale Reference Attack"
    }
    fn player_title(&self) -> &'static str {
        "Crypt 1"
    }
    fn category(&self) -> &'static str {
        "Crypt"
    }
    fn difficulty(&self) -> &'static str {
        "★☆☆"
    }
    fn objective(&self) -> &'static str {
        "Defeat the shielded necromancer."
    }
    fn lesson(&self) -> &'static str {
        "The necromancer's shield was tied to a summon whose reference wasn't cleared on death. Killing the summon first left a stale target the server still accepted damage against. Fix: invalidate references and recompute derived state (shields, buffs) when an entity dies."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Attack { target: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send Attack { target: 2 }\nbatch {\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 50\nsend Attack { target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "crypt",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        has(events, "Attack", "target", 2) && same_tick_count(events, "Attack", "target", 1, 3)
    }
}
