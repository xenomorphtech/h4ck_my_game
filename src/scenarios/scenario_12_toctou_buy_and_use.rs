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
        sprite: "npc",
        x: 4,
        y: 3,
        label: "Apothecary: antidote price 50",
    },
    SceneEntity {
        sprite: "rift",
        x: 5,
        y: 2,
        label: "Poison cloud",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "12-toctou-buy-and-use"
    }
    fn title(&self) -> &'static str {
        "Drink Before Debit: Buy/Use TOCTOU"
    }
    fn player_title(&self) -> &'static str {
        "Apothecary 1"
    }
    fn category(&self) -> &'static str {
        "Apothecary"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Survive the poison cloud with too little gold."
    }
    fn lesson(&self) -> &'static str {
        "The item became usable before payment fully settled, so same-frame use beat the debit."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["BuyItem { item: Int }", "UseItem { item: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send_batch {\n  BuyItem { item: 301 }\n  UseItem { item: 301 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send BuyItem { item: 301 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "apothecary",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events.iter().any(|b| {
            b.name == "BuyItem"
                && field_i64(b, "item") == Some(301)
                && events
                    .iter()
                    .any(|u| u.t == b.t && u.name == "UseItem" && field_i64(u, "item") == Some(301))
        })
    }
}
