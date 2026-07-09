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
        sprite: "chest",
        x: 4,
        y: 2,
        label: "Treasury",
    },
    SceneEntity {
        sprite: "vendor",
        x: 6,
        y: 2,
        label: "Deed vendor: price 500",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "11-currency-integer-overflow"
    }
    fn title(&self) -> &'static str {
        "Gold Wrap: Integer Overflow"
    }
    fn player_title(&self) -> &'static str {
        "Treasury 1"
    }
    fn category(&self) -> &'static str {
        "Treasury"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Buy the castle deed without enough gold."
    }
    fn lesson(&self) -> &'static str {
        "A huge gold operation crossed the integer boundary and wrapped into a usable balance."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["WithdrawGold { amount: Int }", "BuyItem { item: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send WithdrawGold { amount: 2147483647 }\nsend BuyItem { item: 6001 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send BuyItem { item: 6001 }\n"
    }
    fn upcoming(&self) -> bool {
        true
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "treasury",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events.iter().any(|x| {
            x.name == "WithdrawGold" && field_i64(x, "amount").is_some_and(|v| v >= 2_147_483_647)
        }) && has(events, "BuyItem", "item", 6001)
    }
}
