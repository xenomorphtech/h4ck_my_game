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
        label: "Trader",
    },
    SceneEntity {
        sprite: "trade_table",
        x: 3,
        y: 3,
        label: "Trade Window",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "09-dupe-trade-window"
    }
    fn title(&self) -> &'static str {
        "Trade Ghost: Remove After Confirm"
    }
    fn player_title(&self) -> &'static str {
        "Trade 1"
    }
    fn category(&self) -> &'static str {
        "Trade"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Keep your shield and receive the NPC's potion."
    }
    fn lesson(&self) -> &'static str {
        "The trade confirmed from one view of the offer while a same-frame removal changed the live item list."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "OfferItem { trade: Int, item: Int }",
            "RemoveItem { trade: Int, item: Int }",
            "Ready { trade: Int, ready: Bool }",
            "Confirm { trade: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send_batch {\n  OfferItem { trade: 41, item: 2001 }\n  Ready { trade: 41, ready: true }\n  Confirm { trade: 41 }\n  RemoveItem { trade: 41, item: 2001 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send OfferItem { trade: 41, item: 2001 }\nsend Ready { trade: 41, ready: true }\nsend Confirm { trade: 41 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "trade",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        ["OfferItem", "Ready", "Confirm", "RemoveItem"]
            .iter()
            .all(|name| events.iter().any(|x| x.name == *name && x.t == 0))
    }
}
