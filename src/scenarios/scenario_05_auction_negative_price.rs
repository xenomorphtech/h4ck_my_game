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
        sprite: "auction",
        x: 4,
        y: 2,
        label: "Listing #11",
    },
    SceneEntity {
        sprite: "mount",
        x: 5,
        y: 2,
        label: "Phoenix Mount: price 500",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "05-auction-negative-price"
    }
    fn title(&self) -> &'static str {
        "Reverse Bid: Negative Auction Price"
    }
    fn player_title(&self) -> &'static str {
        "Market 1"
    }
    fn category(&self) -> &'static str {
        "Market"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Take home the Phoenix Mount. It costs far more than you have."
    }
    fn lesson(&self) -> &'static str {
        "The purchase deducted the offered price without asserting it was positive, so a negative price credited you gold. Fix: validate numeric bounds (price > 0) server-side before applying any balance change."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["BuyListing { listing: Int, offered_price: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send BuyListing { listing: 11, offered_price: -1 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send BuyListing { listing: 11, offered_price: 500 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "market",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events.iter().any(|x| {
            x.name == "BuyListing"
                && field_i64(x, "listing") == Some(11)
                && field_i64(x, "offered_price").is_some_and(|v| v < 0)
        })
    }
}
