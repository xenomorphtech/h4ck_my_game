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
        sprite: "wand",
        x: 2,
        y: 3,
        label: "PowerStrike crystal #10 — 50 shield damage, 1000ms cooldown",
    },
    SceneEntity {
        sprite: "boss",
        x: 4,
        y: 2,
        label: "Shielded Boss #1 (1 HP) — shield 150, retaliates in 500ms",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];
const BOSS_ID: i64 = 1;
const POWER_STRIKE_ID: i64 = 10;
const POWER_STRIKE_COOLDOWN_MS: u64 = 1000;
const ATTACK_COOLDOWN_MS: u64 = 750;

fn cooldown_info(
    t: u64,
    action: &str,
    target: i64,
    skill: Option<i64>,
    ready_at: u64,
) -> PacketEvent {
    let mut fields = Map::from_iter([
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
    ]);
    if let Some(skill) = skill {
        fields.insert("skill".to_string(), json!(skill));
    }

    PacketEvent {
        t,
        kind: "server".to_string(),
        name: "Info".to_string(),
        fields,
    }
}

fn cooldown_notifications(events: &[ClientEvent]) -> Vec<PacketEvent> {
    let mut actions: Vec<&ClientEvent> = events
        .iter()
        .filter(|event| {
            (event.name == "CastSkill"
                && field_i64(event, "skill") == Some(POWER_STRIKE_ID)
                && field_i64(event, "target") == Some(BOSS_ID))
                || (event.name == "Attack" && field_i64(event, "target") == Some(BOSS_ID))
        })
        .collect();
    actions.sort_by_key(|event| event.t);

    let mut out = Vec::new();
    let mut skill_ready_at = 0;
    let mut attack_ready_at = 0;
    let mut idx = 0;

    while idx < actions.len() {
        let t = actions[idx].t;
        let mut skill_accepted_this_tick = false;
        let mut attack_accepted_this_tick = false;

        while idx < actions.len() && actions[idx].t == t {
            let event = actions[idx];
            if event.name == "CastSkill" {
                if t < skill_ready_at {
                    out.push(cooldown_info(
                        t,
                        "PowerStrike",
                        BOSS_ID,
                        Some(POWER_STRIKE_ID),
                        skill_ready_at,
                    ));
                } else {
                    skill_accepted_this_tick = true;
                }
            } else if t < attack_ready_at {
                out.push(cooldown_info(t, "Attack", BOSS_ID, None, attack_ready_at));
            } else {
                attack_accepted_this_tick = true;
            }
            idx += 1;
        }

        if skill_accepted_this_tick {
            skill_ready_at = t.saturating_add(POWER_STRIKE_COOLDOWN_MS);
        }
        if attack_accepted_this_tick {
            attack_ready_at = t.saturating_add(ATTACK_COOLDOWN_MS);
        }
    }

    out
}

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "16-cooldown-bypass-batch"
    }
    fn title(&self) -> &'static str {
        "No Cooldown Yet: Batched Skill Spam"
    }
    fn player_title(&self) -> &'static str {
        "Arena 3"
    }
    fn category(&self) -> &'static str {
        "Arena"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Break the boss shield, then finish the 1 HP boss before it retaliates. Shield: 150. PowerStrike: 50 shield damage, 1000ms cooldown. Attack: 10 HP damage, 750ms cooldown. Retaliation: 500ms."
    }
    fn lesson(&self) -> &'static str {
        "Cooldowns were checked before the frame and written after all frame actions, so repeated same-frame casts all saw the skill as ready. Fix: reserve cooldown immediately before applying the first skill effect."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "CastSkill { skill: Int, target: Int }",
            "Attack { target: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "batch {\n  send CastSkill { skill: 10, target: 1 }\n  send CastSkill { skill: 10, target: 1 }\n  send CastSkill { skill: 10, target: 1 }\n  send Attack { target: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send CastSkill { skill: 10, target: 1 }\nsleep 1000\nsend CastSkill { skill: 10, target: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "arena",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        // The shield (150) needs three PowerStrikes (50 each) inside one tick,
        // and the boss (1 HP) must be finished by an Attack landing in that same
        // tick — before the 500ms retaliation resolves.
        events.iter().any(|anchor| {
            anchor.name == "CastSkill"
                && field_i64(anchor, "skill") == Some(10)
                && same_tick_count(events, "CastSkill", "skill", 10, 3)
                && events.iter().any(|event| {
                    event.t == anchor.t
                        && event.name == "Attack"
                        && field_i64(event, "target") == Some(1)
                })
        })
    }
    fn notifications(&self, events: &[ClientEvent]) -> Vec<PacketEvent> {
        cooldown_notifications(events)
    }
}
