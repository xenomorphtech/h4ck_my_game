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
        sprite: "gem",
        x: 5,
        y: 2,
        label: "Gem x1, listing #21",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "06-auction-buyout-race"
    }
    fn title(&self) -> &'static str {
        "Last Copy: Auction Buyout Race"
    }
    fn player_title(&self) -> &'static str {
        "Market 2"
    }
    fn category(&self) -> &'static str {
        "Market"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Obtain two gems. Only one is for sale."
    }
    fn lesson(&self) -> &'static str {
        "Two buyouts shared the same availability window, so both claimed the last listed item before stock changed."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Buyout { listing: Int, quantity: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send_batch {\n  Buyout { listing: 21, quantity: 1 }\n  Buyout { listing: 21, quantity: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Buyout { listing: 21, quantity: 1 }\nsleep 20\nsend Buyout { listing: 21, quantity: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "market",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events
            .iter()
            .filter(|x| {
                x.name == "Buyout"
                    && field_i64(x, "listing") == Some(21)
                    && field_i64(x, "quantity") == Some(1)
            })
            .map(|x| x.t)
            .any(|t| {
                events
                    .iter()
                    .filter(|x| {
                        x.t == t
                            && x.name == "Buyout"
                            && field_i64(x, "listing") == Some(21)
                            && field_i64(x, "quantity") == Some(1)
                    })
                    .count()
                    >= 2
            })
    }
}
