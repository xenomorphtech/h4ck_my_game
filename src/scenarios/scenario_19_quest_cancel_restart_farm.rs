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
        sprite: "quest_giver",
        x: 4,
        y: 3,
        label: "Quartermaster: accept grants a kit",
    },
    SceneEntity {
        sprite: "crate",
        x: 6,
        y: 2,
        label: "Provision depot",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "19-quest-cancel-restart-farm"
    }
    fn title(&self) -> &'static str {
        "Starter Kit Farm: Cancel and Restart"
    }
    fn player_title(&self) -> &'static str {
        "Guild 2"
    }
    fn category(&self) -> &'static str {
        "Guild"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Collect five provision kits."
    }
    fn lesson(&self) -> &'static str {
        "Quest restart state granted starter kits repeatedly while previous grants remained in inventory."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["AcceptQuest { quest: Int }", "AbandonQuest { quest: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "for i in 1..6 {\n  send AcceptQuest { quest: 71 }\n  send AbandonQuest { quest: 71 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send AcceptQuest { quest: 71 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "guild",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events
            .iter()
            .filter(|x| x.name == "AcceptQuest" && field_i64(x, "quest") == Some(71))
            .count()
            >= 5
            && events
                .iter()
                .filter(|x| x.name == "AbandonQuest" && field_i64(x, "quest") == Some(71))
                .count()
                >= 5
    }
}
