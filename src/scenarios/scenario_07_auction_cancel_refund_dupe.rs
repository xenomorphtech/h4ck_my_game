use super::rules::has;
use super::{BlockedTile, Scenario, Scene, SceneEntity};
use crate::engine::{field_i64, ClientEvent};
use crate::protocol::PacketEvent;
use serde_json::{json, Map};

pub struct ScenarioImpl;

pub static SCENARIO: ScenarioImpl = ScenarioImpl;

const ENTITIES: &[SceneEntity] = &[SceneEntity {
    sprite: "hero",
    x: 1,
    y: 3,
    label: "You",
}];
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
        "send CancelListing { listing: 32 }\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send CancelListing { listing: 31 }\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "market",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        // The sold sword listing double-settles: canceling listing #32 returns
        // the sword (creates mail #2) while the completed sale still pays out
        // (mail #1). Listing #31 is just the visible UI cancel affordance and is
        // a different item, so canceling it cannot satisfy the sword objective.
        let cancel_idx = events
            .iter()
            .position(|x| x.name == "CancelListing" && field_i64(x, "listing") == Some(32));
        let Some(cancel_idx) = cancel_idx else {
            return false;
        };
        // Mail #2 (returned sword) only exists once the cancel has been sent, so
        // it must be claimed after the cancel -- claiming it early is invalid.
        let claimed_return_after_cancel = events
            .iter()
            .skip(cancel_idx + 1)
            .any(|x| x.name == "ClaimMailbox" && field_i64(x, "mail") == Some(2));
        claimed_return_after_cancel && has(events, "ClaimMailbox", "mail", 1)
    }
    fn notifications(&self, events: &[ClientEvent]) -> Vec<PacketEvent> {
        // The server owns market/mail state. When the player cancels a listing,
        // the world removes that listing and mails the returned item back. The
        // client renders these authoritative events; it never invents them.
        let mut out = Vec::new();
        for event in events {
            if event.name != "CancelListing" {
                continue;
            }
            let Some(listing) = field_i64(event, "listing") else {
                continue;
            };
            // Each listing returns its own item. Listing #31 is the visible
            // decoy (a Copper Charm); listing #32 is the sold sword whose cancel
            // path double-settles the exploit.
            let (item, sprite, mail_id) = match listing {
                31 => ("Copper Charm", "gem", 3),
                32 => ("Listed Sword", "blade", 2),
                _ => continue,
            };
            out.push(PacketEvent {
                t: event.t,
                kind: "server".to_string(),
                name: "ListingRemoved".to_string(),
                fields: Map::from_iter([("listing".to_string(), json!(listing))]),
            });
            out.push(PacketEvent {
                t: event.t,
                kind: "server".to_string(),
                name: "MailCreated".to_string(),
                fields: Map::from_iter([
                    ("mail".to_string(), json!(mail_id)),
                    ("subject".to_string(), json!("Listing cancelled")),
                    ("attachment".to_string(), json!(item)),
                    ("sprite".to_string(), json!(sprite)),
                    ("status".to_string(), json!("unread")),
                ]),
            });
        }
        out
    }
}
