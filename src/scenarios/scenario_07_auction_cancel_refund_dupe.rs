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
        label: "Listing #31: buyer settles at t=300",
    },
    SceneEntity {
        sprite: "mailbox",
        x: 6,
        y: 3,
        label: "Mailbox: sale mail #1, cancel mail #2",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "07-auction-cancel-refund-dupe"
    }
    fn title(&self) -> &'static str {
        "Sold and Cancelled: Auction Refund Dupe"
    }
    fn player_title(&self) -> &'static str {
        "Market 3"
    }
    fn category(&self) -> &'static str {
        "Market"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Walk away holding both the listed sword and the gold it sold for."
    }
    fn lesson(&self) -> &'static str {
        "A cancel that landed after the sale settled refunded the item while the buyer kept the payout. The cancel path didn't recheck sale state at commit time. Fix: re-validate the listing's current state inside the transaction and reject cancels on already-settled sales."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "CancelListing { listing: Int }",
            "ClaimMailbox { mail: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "at(300) {\n  send CancelListing { listing: 31 }\n}\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send CancelListing { listing: 31 }\nsleep 300\nsend ClaimMailbox { mail: 1 }\n"
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
            .any(|x| x.name == "CancelListing" && x.t == 300 && field_i64(x, "listing") == Some(31))
            && has(events, "ClaimMailbox", "mail", 1)
            && has(events, "ClaimMailbox", "mail", 2)
    }
}
