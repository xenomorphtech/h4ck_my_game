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
        sprite: "wand",
        x: 2,
        y: 3,
        label: "Skill crystal #10",
    },
    SceneEntity {
        sprite: "boss",
        x: 4,
        y: 2,
        label: "Shielded Boss #1",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "16-cooldown-bypass-batch"
    }
    fn title(&self) -> &'static str {
        "No Cooldown Yet: Batched Skill Spam"
    }
    fn player_title(&self) -> &'static str {
        "Arena 3"
    }
    fn category(&self) -> &'static str {
        "Arena"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Break the boss shield."
    }
    fn lesson(&self) -> &'static str {
        "Cooldowns were checked before the frame and written after all frame actions, so repeated same-frame casts all saw the skill as ready. Fix: reserve cooldown immediately before applying the first skill effect."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["CastSkill { skill: Int, target: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  send CastSkill { skill: 10, target: 1 }\n  send CastSkill { skill: 10, target: 1 }\n  send CastSkill { skill: 10, target: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send CastSkill { skill: 10, target: 1 }\nsleep 1000\nsend CastSkill { skill: 10, target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "arena",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        same_tick_count(events, "CastSkill", "skill", 10, 3)
    }
}
