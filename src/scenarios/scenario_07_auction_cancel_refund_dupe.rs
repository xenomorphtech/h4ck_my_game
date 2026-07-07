use super::{BlockedTile, Scenario, Scene, SceneEntity};
use crate::engine::{field_i64, ClientEvent};
use crate::protocol::PacketEvent;
use serde_json::{json, Map};
use std::collections::{BTreeMap, BTreeSet};

pub struct ScenarioImpl;

pub static SCENARIO: ScenarioImpl = ScenarioImpl;

const ENTITIES: &[SceneEntity] = &[SceneEntity {
    sprite: "hero",
    x: 1,
    y: 3,
    label: "You",
}];
const BLOCKED_TILES: &[BlockedTile] = &[];

#[derive(Clone, Copy)]
struct MailReward {
    item: &'static str,
    sprite: &'static str,
    quantity: i64,
    slot: &'static str,
}

const SALE_PROCEEDS: MailReward = MailReward {
    item: "Gold",
    sprite: "currency",
    quantity: 300,
    slot: "wallet",
};

fn cancel_reward(listing: i64) -> Option<MailReward> {
    match listing {
        31 => Some(MailReward {
            item: "Copper Charm",
            sprite: "gem",
            quantity: 1,
            slot: "bag",
        }),
        32 => Some(MailReward {
            item: "Listed Sword",
            sprite: "blade",
            quantity: 1,
            slot: "auction",
        }),
        _ => None,
    }
}

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "07-auction-cancel-refund-dupe"
    }
    fn title(&self) -> &'static str {
        "Sold and Cancelled: Auction Refund Dupe"
    }
    fn player_title(&self) -> &'static str {
        "market-3"
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
        "The sale payout and cancel refund both landed, leaving you with the gold and the returned item."
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
        "send ClaimMailbox { mail: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "market",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        let mut next_mail = 2;
        let mut removed = BTreeSet::new();
        let mut available = BTreeMap::from([(1, SALE_PROCEEDS)]);
        let mut claimed = BTreeSet::new();
        let mut claimed_gold = false;
        let mut claimed_sword = false;

        for event in events {
            if event.name == "CancelListing" {
                if let Some(listing) = field_i64(event, "listing") {
                    if removed.insert(listing) {
                        if let Some(reward) = cancel_reward(listing) {
                            available.insert(next_mail, reward);
                            next_mail += 1;
                        }
                    }
                }
            } else if event.name == "ClaimMailbox" {
                let Some(mail) = field_i64(event, "mail") else {
                    continue;
                };
                let Some(reward) = available.get(&mail) else {
                    continue;
                };
                if !claimed.insert(mail) {
                    continue;
                }
                if reward.item == "Gold" {
                    claimed_gold = true;
                } else if reward.item == "Listed Sword" {
                    claimed_sword = true;
                }
            }
        }

        claimed_gold && claimed_sword
    }
    fn notifications(&self, events: &[ClientEvent]) -> Vec<PacketEvent> {
        // The server owns market/mail state. When the player cancels a listing,
        // the world removes that listing and mails the returned item back. The
        // client renders these authoritative events; it never invents them.
        let mut out = Vec::new();
        let mut next_mail = 2;
        let mut removed = BTreeSet::new();
        let mut available = BTreeMap::from([(1, SALE_PROCEEDS)]);
        let mut claimed = BTreeSet::new();

        for event in events {
            if event.name == "CancelListing" {
                let Some(listing) = field_i64(event, "listing") else {
                    continue;
                };
                if !removed.insert(listing) {
                    continue;
                }
                let Some(reward) = cancel_reward(listing) else {
                    continue;
                };
                let mail_id = next_mail;
                next_mail += 1;
                available.insert(mail_id, reward);

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
                        ("attachment".to_string(), json!(reward.item)),
                        ("sprite".to_string(), json!(reward.sprite)),
                        ("status".to_string(), json!("unclaimed")),
                    ]),
                });
            } else if event.name == "ClaimMailbox" {
                let Some(mail) = field_i64(event, "mail") else {
                    out.push(PacketEvent {
                        t: event.t,
                        kind: "server".to_string(),
                        name: "MailClaimFailed".to_string(),
                        fields: Map::from_iter([("reason".to_string(), json!("invalid_mail"))]),
                    });
                    continue;
                };
                let Some(reward) = available.get(&mail) else {
                    out.push(PacketEvent {
                        t: event.t,
                        kind: "server".to_string(),
                        name: "MailClaimFailed".to_string(),
                        fields: Map::from_iter([
                            ("mail".to_string(), json!(mail)),
                            ("reason".to_string(), json!("not_found")),
                        ]),
                    });
                    continue;
                };
                if !claimed.insert(mail) {
                    out.push(PacketEvent {
                        t: event.t,
                        kind: "server".to_string(),
                        name: "MailClaimFailed".to_string(),
                        fields: Map::from_iter([
                            ("mail".to_string(), json!(mail)),
                            ("reason".to_string(), json!("already_claimed")),
                        ]),
                    });
                    continue;
                }
                out.push(PacketEvent {
                    t: event.t,
                    kind: "server".to_string(),
                    name: "MailClaimed".to_string(),
                    fields: Map::from_iter([
                        ("mail".to_string(), json!(mail)),
                        ("status".to_string(), json!("claimed")),
                    ]),
                });
                out.push(PacketEvent {
                    t: event.t,
                    kind: "server".to_string(),
                    name: "InventoryAdded".to_string(),
                    fields: Map::from_iter([
                        ("item".to_string(), json!(reward.item)),
                        ("sprite".to_string(), json!(reward.sprite)),
                        ("quantity".to_string(), json!(reward.quantity)),
                        ("slot".to_string(), json!(reward.slot)),
                    ]),
                });
            }
        }
        out
    }
}
