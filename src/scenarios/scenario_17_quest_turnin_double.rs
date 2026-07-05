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
        label: "Quest Giver: quest #61",
    },
    SceneEntity {
        sprite: "chest",
        x: 6,
        y: 2,
        label: "Reward Chests 0/2",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "17-quest-turnin-double"
    }
    fn title(&self) -> &'static str {
        "Twice for One Head: Double Quest Turn-In"
    }
    fn player_title(&self) -> &'static str {
        "Guild 1"
    }
    fn category(&self) -> &'static str {
        "Guild"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Claim two rewards for one quest item."
    }
    fn lesson(&self) -> &'static str {
        "The turn-in queued reward creation before marking the quest complete. Same-frame requests both saw the quest as unfinished. Fix: make turn-in idempotent and mark completion atomically with reward creation."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["TurnInQuest { quest: Int, item: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  send TurnInQuest { quest: 61, item: 4001 }\n  send TurnInQuest { quest: 61, item: 4001 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send TurnInQuest { quest: 61, item: 4001 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "guild",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        same_tick_count(events, "TurnInQuest", "quest", 61, 2)
    }
}
