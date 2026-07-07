use super::rules::same_tick_count;
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
        label: "Monster #1 (120 HP)",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];
const PLAYER_ID: i64 = 0;
const MONSTER_ID: i64 = 1;
const MONSTER_HP: i64 = 120;
const ATTACK_DAMAGE: i64 = 40;
const RETALIATION_DAMAGE: i64 = 999;
const RETALIATION_DELAY_MS: u64 = 250;

fn damage_event(t: u64, source: i64, target: i64, amount: i64) -> PacketEvent {
    PacketEvent {
        t,
        kind: "server".to_string(),
        name: "Damage".to_string(),
        fields: Map::from_iter([
            ("source".to_string(), json!(source)),
            ("target".to_string(), json!(target)),
            ("amount".to_string(), json!(amount)),
        ]),
    }
}

fn death_event(t: u64, target: i64) -> PacketEvent {
    PacketEvent {
        t,
        kind: "server".to_string(),
        name: "Death".to_string(),
        fields: Map::from_iter([("target".to_string(), json!(target))]),
    }
}

fn combat_notifications(events: &[ClientEvent]) -> Vec<PacketEvent> {
    let mut attacks: Vec<&ClientEvent> = events
        .iter()
        .filter(|event| event.name == "Attack" && field_i64(event, "target") == Some(MONSTER_ID))
        .collect();
    attacks.sort_by_key(|event| event.t);

    let mut out = Vec::new();
    let mut monster_hp = MONSTER_HP;
    let mut idx = 0;

    while idx < attacks.len() {
        let t = attacks[idx].t;

        while idx < attacks.len() && attacks[idx].t == t {
            let damage = field_i64(attacks[idx], "power").unwrap_or(ATTACK_DAMAGE);
            monster_hp -= damage;
            out.push(damage_event(t, PLAYER_ID, MONSTER_ID, damage));
            idx += 1;

            if monster_hp <= 0 {
                out.push(death_event(t, MONSTER_ID));
                return out;
            }
        }

        let retaliation_t = t.saturating_add(RETALIATION_DELAY_MS);
        out.push(damage_event(
            retaliation_t,
            MONSTER_ID,
            PLAYER_ID,
            RETALIATION_DAMAGE,
        ));
        out.push(death_event(retaliation_t, PLAYER_ID));
        return out;
    }

    out
}

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "01-first-blood-batch"
    }
    fn title(&self) -> &'static str {
        "First Blood: Batch the Fatal Window"
    }
    fn player_title(&self) -> &'static str {
        "Arena 1"
    }
    fn category(&self) -> &'static str {
        "Arena"
    }
    fn difficulty(&self) -> &'static str {
        "★☆☆"
    }
    fn objective(&self) -> &'static str {
        "Kill the monster."
    }
    fn lesson(&self) -> &'static str {
        "Three attacks landed in the same packet frame, collapsing the retaliation window before the monster could answer."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["Attack { target: Int, power: Int = 40 }"]
    }
    fn solution_script(&self) -> &'static str {
        "send_batch {\n  Attack { target: 1 }\n  Attack { target: 1 }\n  Attack { target: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send Attack { target: 1 }\nsleep 100\nsend Attack { target: 1 }\nsleep 100\nsend Attack { target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "arena",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        same_tick_count(events, "Attack", "target", 1, 3)
    }
    fn notifications(&self, events: &[ClientEvent]) -> Vec<PacketEvent> {
        combat_notifications(events)
    }
}
