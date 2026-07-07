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
const PLAYER_ID: i64 = 0;
const BOSS_ID: i64 = 1;
const BOSS_HP: i64 = 1;
const SHIELD_HP: i64 = 150;
const POWER_STRIKE_ID: i64 = 10;
const POWER_STRIKE_DAMAGE: i64 = 50;
const ATTACK_DAMAGE: i64 = 10;
const RETALIATION_DAMAGE: i64 = 999;
const POWER_STRIKE_COOLDOWN_MS: u64 = 1000;
const ATTACK_COOLDOWN_MS: u64 = 750;
const RETALIATION_DELAY_MS: u64 = 500;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionKind {
    PowerStrike,
    Attack,
}

struct ResolvedAction {
    t: u64,
    kind: ActionKind,
    cooldown_ready_at: Option<u64>,
}

fn cooldown_info(t: u64, kind: ActionKind, target: i64, ready_at: u64) -> PacketEvent {
    let (action, skill) = match kind {
        ActionKind::PowerStrike => ("PowerStrike", Some(POWER_STRIKE_ID)),
        ActionKind::Attack => ("Attack", None),
    };
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

fn action_kind(event: &ClientEvent) -> Option<ActionKind> {
    if event.name == "CastSkill"
        && field_i64(event, "skill") == Some(POWER_STRIKE_ID)
        && field_i64(event, "target") == Some(BOSS_ID)
    {
        Some(ActionKind::PowerStrike)
    } else if event.name == "Attack" && field_i64(event, "target") == Some(BOSS_ID) {
        Some(ActionKind::Attack)
    } else {
        None
    }
}

fn simulate(events: &[ClientEvent]) -> Vec<ResolvedAction> {
    let mut actions: Vec<(usize, &ClientEvent, ActionKind)> = events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| {
            let kind = action_kind(event)?;
            Some((idx, event, kind))
        })
        .collect();
    actions.sort_by_key(|(idx, event, _)| (event.t, *idx));

    let mut resolved = Vec::new();
    let mut skill_ready_at = 0;
    let mut attack_ready_at = 0;
    let mut idx = 0;

    while idx < actions.len() {
        let t = actions[idx].1.t;
        let mut skill_accepted_this_tick = false;
        let mut attack_accepted_this_tick = false;

        while idx < actions.len() && actions[idx].1.t == t {
            let kind = actions[idx].2;
            match kind {
                ActionKind::PowerStrike if t < skill_ready_at => {
                    resolved.push(ResolvedAction {
                        t,
                        kind,
                        cooldown_ready_at: Some(skill_ready_at),
                    });
                }
                ActionKind::PowerStrike => {
                    skill_accepted_this_tick = true;
                    resolved.push(ResolvedAction {
                        t,
                        kind,
                        cooldown_ready_at: None,
                    });
                }
                ActionKind::Attack if t < attack_ready_at => {
                    resolved.push(ResolvedAction {
                        t,
                        kind,
                        cooldown_ready_at: Some(attack_ready_at),
                    });
                }
                ActionKind::Attack => {
                    attack_accepted_this_tick = true;
                    resolved.push(ResolvedAction {
                        t,
                        kind,
                        cooldown_ready_at: None,
                    });
                }
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

    resolved
}

fn boss_killed(actions: &[ResolvedAction]) -> bool {
    let mut shield_hp = SHIELD_HP;
    let mut boss_hp = BOSS_HP;
    let mut retaliation_at: Option<u64> = None;
    let mut idx = 0;

    while idx < actions.len() {
        let t = actions[idx].t;
        if retaliation_at.is_some_and(|due| due <= t) {
            return false;
        }

        let mut accepted_this_tick = false;
        while idx < actions.len() && actions[idx].t == t {
            let action = &actions[idx];
            idx += 1;
            if action.cooldown_ready_at.is_some() {
                continue;
            }

            accepted_this_tick = true;
            match action.kind {
                ActionKind::PowerStrike => {
                    shield_hp = shield_hp.saturating_sub(POWER_STRIKE_DAMAGE);
                }
                ActionKind::Attack if shield_hp <= 0 => {
                    boss_hp -= ATTACK_DAMAGE;
                }
                ActionKind::Attack => {}
            }

            if boss_hp <= 0 {
                return true;
            }
        }

        if accepted_this_tick && retaliation_at.is_none() {
            retaliation_at = Some(t.saturating_add(RETALIATION_DELAY_MS));
        }
    }

    false
}

fn combat_notifications(actions: &[ResolvedAction]) -> Vec<PacketEvent> {
    let mut out = Vec::new();
    let mut shield_hp = SHIELD_HP;
    let mut boss_hp = BOSS_HP;
    let mut player_dead = false;
    let mut retaliation_at: Option<u64> = None;
    let mut idx = 0;

    while idx < actions.len() {
        let t = actions[idx].t;
        if !player_dead && retaliation_at.is_some_and(|due| due <= t) {
            let due = retaliation_at.unwrap();
            out.push(damage_event(due, BOSS_ID, PLAYER_ID, RETALIATION_DAMAGE));
            out.push(death_event(due, PLAYER_ID));
            player_dead = true;
        }

        let mut accepted_this_tick = false;
        while idx < actions.len() && actions[idx].t == t {
            let action = &actions[idx];
            idx += 1;

            if let Some(ready_at) = action.cooldown_ready_at {
                out.push(cooldown_info(t, action.kind, BOSS_ID, ready_at));
                continue;
            }
            if player_dead || boss_hp <= 0 {
                continue;
            }

            accepted_this_tick = true;
            match action.kind {
                ActionKind::PowerStrike => {
                    shield_hp = shield_hp.saturating_sub(POWER_STRIKE_DAMAGE);
                    out.push(damage_event(t, PLAYER_ID, BOSS_ID, POWER_STRIKE_DAMAGE));
                }
                ActionKind::Attack if shield_hp <= 0 => {
                    boss_hp -= ATTACK_DAMAGE;
                    out.push(damage_event(t, PLAYER_ID, BOSS_ID, ATTACK_DAMAGE));
                    if boss_hp <= 0 {
                        out.push(death_event(t, BOSS_ID));
                    }
                }
                ActionKind::Attack => {
                    out.push(damage_event(t, PLAYER_ID, BOSS_ID, ATTACK_DAMAGE));
                }
            }
        }

        if boss_hp <= 0 {
            break;
        }
        if accepted_this_tick && retaliation_at.is_none() {
            retaliation_at = Some(t.saturating_add(RETALIATION_DELAY_MS));
        }
    }

    if !player_dead && boss_hp > 0 {
        if let Some(due) = retaliation_at {
            out.push(damage_event(due, BOSS_ID, PLAYER_ID, RETALIATION_DAMAGE));
            out.push(death_event(due, PLAYER_ID));
        }
    }

    out.sort_by_key(|event| event.t);
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
        "Multiple same-frame casts all saw the cooldown as ready, breaking the shield before the boss could retaliate."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "CastSkill { skill: Int, target: Int }",
            "Attack { target: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send_batch {\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  Attack { target: 1 }\n}\n"
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
        boss_killed(&simulate(events))
    }
    fn notifications(&self, events: &[ClientEvent]) -> Vec<PacketEvent> {
        combat_notifications(&simulate(events))
    }
}
