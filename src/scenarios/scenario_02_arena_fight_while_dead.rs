use super::{BlockedTile, Scenario, Scene, SceneEntity};
use crate::engine::{field_i64, ClientEvent};
use crate::protocol::PacketEvent;
use serde_json::{json, Map};

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
        sprite: "monster",
        x: 4,
        y: 2,
        label: "Arena Monster #1 (160 HP)",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];
const PLAYER_ID: i64 = 0;
const MONSTER_ID: i64 = 1;
const MONSTER_HP: i64 = 160;
const ATTACK_DAMAGE: i64 = 40;
/// Minimum time between two attacks that actually land. Attacks that arrive
/// sooner (including several in the same tick via `batch`) are on cooldown and
/// are ignored by the server, so batching does not help here.
const ATTACK_COOLDOWN_MS: u64 = 750;
/// How long after an attack the monster's counterattack resolves.
const RETALIATION_DELAY_MS: u64 = 250;

/// A single attack the server actually accepted (i.e. it was not on cooldown),
/// with the virtual time it landed and the damage it dealt.
struct LandedAttack {
    t: u64,
    damage: i64,
    cooldown_ready_at: Option<u64>,
}

/// Replay the player's Attack packets against the monster, enforcing the attack
/// cooldown. Returns the attacks that actually landed, in order. This is the one
/// authoritative combat model; both `check_win` and `notifications` use it so
/// the rendered feed and the win condition can never disagree.
fn cooldown_info(t: u64, action: &str, target: i64, ready_at: u64) -> PacketEvent {
    PacketEvent {
        t,
        kind: "server".to_string(),
        name: "Info".to_string(),
        fields: Map::from_iter([
            ("level".to_string(), json!("error")),
            ("reason".to_string(), json!("cooldown")),
            ("action".to_string(), json!(action)),
            ("target".to_string(), json!(target)),
            ("ready_at_ms".to_string(), json!(ready_at)),
            (
                "remaining_ms".to_string(),
                json!(ready_at.saturating_sub(t)),
            ),
            (
                "message".to_string(),
                json!(format!("{action} is still on cooldown")),
            ),
        ]),
    }
}

fn simulate(events: &[ClientEvent]) -> Vec<LandedAttack> {
    let mut attacks: Vec<&ClientEvent> = events
        .iter()
        .filter(|x| x.name == "Attack" && field_i64(x, "target") == Some(MONSTER_ID))
        .collect();
    attacks.sort_by_key(|x| x.t);

    let mut landed = Vec::new();
    let mut last_landed: Option<u64> = None;
    for attack in attacks {
        let ready = match last_landed {
            None => true,
            Some(prev) => attack.t.saturating_sub(prev) >= ATTACK_COOLDOWN_MS,
        };
        if !ready {
            let ready_at = last_landed
                .unwrap_or(attack.t)
                .saturating_add(ATTACK_COOLDOWN_MS);
            landed.push(LandedAttack {
                t: attack.t,
                damage: 0,
                cooldown_ready_at: Some(ready_at),
            });
            continue;
        }
        last_landed = Some(attack.t);
        landed.push(LandedAttack {
            t: attack.t,
            damage: field_i64(attack, "power").unwrap_or(ATTACK_DAMAGE),
            cooldown_ready_at: None,
        });
    }
    landed
}

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "02-arena-fight-while-dead"
    }
    fn title(&self) -> &'static str {
        "Second Wind: Dead Player Action Accepted"
    }
    fn player_title(&self) -> &'static str {
        "Arena 2"
    }
    fn category(&self) -> &'static str {
        "Arena"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Kill the monster."
    }
    fn lesson(&self) -> &'static str {
        "The server kept accepting attacks after your death, so the monster could still be finished from a dead-player state."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Attack { target: Int, power: Int = 40 }"]
    }
    fn solution_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 750\nsend Attack { target: 1 }\nsleep 750\nsend Attack { target: 1 }\nsleep 750\nsend Attack { target: 1 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 300\nsend Attack { target: 1 }\nsleep 100\nsend Attack { target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "arena",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        // The win condition is simply: the monster is dead. Sum the damage from
        // every attack the server accepted (cooldown-gated, order-independent).
        // How the packets were grouped (batched, spaced) does not matter beyond
        // the cooldown the simulation already enforces.
        let dealt: i64 = simulate(events)
            .iter()
            .filter(|a| a.cooldown_ready_at.is_none())
            .map(|a| a.damage)
            .sum();
        dealt >= MONSTER_HP
    }
    fn notifications(&self, events: &[ClientEvent]) -> Vec<PacketEvent> {
        // The server is authoritative for combat. It emits the damage each
        // landed attack deals, the fatal counterattack that kills the player,
        // and the monster's death. The client renders these events; it must not
        // invent retaliation or death on its own.
        let resolved = simulate(events);
        let mut out = Vec::new();
        let mut monster_hp = MONSTER_HP;
        let mut player_dead = false;

        for attack in &resolved {
            if let Some(ready_at) = attack.cooldown_ready_at {
                out.push(cooldown_info(attack.t, "Attack", MONSTER_ID, ready_at));
                continue;
            }
            if monster_hp <= 0 {
                break;
            }
            monster_hp -= attack.damage;
            out.push(PacketEvent {
                t: attack.t,
                kind: "server".to_string(),
                name: "Damage".to_string(),
                fields: Map::from_iter([
                    ("source".to_string(), json!(PLAYER_ID)),
                    ("target".to_string(), json!(MONSTER_ID)),
                    ("amount".to_string(), json!(attack.damage)),
                ]),
            });

            if monster_hp <= 0 {
                // The monster dies from this hit; no counterattack follows.
                out.push(PacketEvent {
                    t: attack.t,
                    kind: "server".to_string(),
                    name: "Death".to_string(),
                    fields: Map::from_iter([("target".to_string(), json!(MONSTER_ID))]),
                });
                break;
            }

            // The monster survives and counterattacks. The first counterattack
            // is fatal to the player — but the buggy handler keeps accepting the
            // dead player's later attacks, which is the whole point of Arena 2.
            if !player_dead {
                player_dead = true;
                let t = attack.t + RETALIATION_DELAY_MS;
                out.push(PacketEvent {
                    t,
                    kind: "server".to_string(),
                    name: "Damage".to_string(),
                    fields: Map::from_iter([
                        ("source".to_string(), json!(MONSTER_ID)),
                        ("target".to_string(), json!(PLAYER_ID)),
                        ("amount".to_string(), json!(999)),
                    ]),
                });
                out.push(PacketEvent {
                    t,
                    kind: "server".to_string(),
                    name: "Death".to_string(),
                    fields: Map::from_iter([("target".to_string(), json!(PLAYER_ID))]),
                });
            }
        }

        out.sort_by_key(|event| event.t);
        out
    }
}
