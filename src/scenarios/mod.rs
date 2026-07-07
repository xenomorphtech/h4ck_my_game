use crate::engine::ClientEvent;
use crate::protocol::PacketEvent;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

pub mod rules;
mod scenario_01_first_blood_batch;
mod scenario_02_arena_fight_while_dead;
mod scenario_02_target_validation_range;
mod scenario_04_target_validation_faction;
mod scenario_05_auction_negative_price;
mod scenario_06_auction_buyout_race;
mod scenario_07_auction_cancel_refund_dupe;
mod scenario_08_dupe_mail_desync;
mod scenario_09_dupe_trade_window;
mod scenario_10_dupe_stack_split_negative;
mod scenario_11_currency_integer_overflow;
mod scenario_12_toctou_buy_and_use;
mod scenario_13_rate_limit_timestamp;
mod scenario_14_rollback_move_teleport;
mod scenario_15_replay_signed_loot;
mod scenario_16_cooldown_bypass_batch;
mod scenario_17_quest_turnin_double;
mod scenario_18_instanced_loot_ownership;
mod scenario_19_quest_cancel_restart_farm;
mod scenario_20_chest_multi_interaction_dupe;
mod scenario_21_telehacking_position_spoof;
mod scenario_22_crafting_clientside_materials;

/// Broad gameplay classification for a scene entity. Derived from the sprite so
/// existing scenario definitions stay compact, but exposed to the client so the
/// renderer can treat, e.g., every `monster` the same way on any map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Player,
    Monster,
    Npc,
    Structure,
    Item,
    Prop,
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Player => "player",
            EntityKind::Monster => "monster",
            EntityKind::Npc => "npc",
            EntityKind::Structure => "structure",
            EntityKind::Item => "item",
            EntityKind::Prop => "prop",
        }
    }
}

/// Behavior shared by everything drawn in a scene. Implemented as a trait so new
/// entity carriers (not just the static `SceneEntity`) can plug into the same
/// rendering/serialization contract, and so per-kind behavior (monsters expose
/// combat stats, structures block movement, etc.) lives in one place.
pub trait Entity {
    fn sprite(&self) -> &str;
    fn label(&self) -> &str;
    fn x(&self) -> i32;
    fn y(&self) -> i32;

    /// Gameplay classification, derived from the sprite by default.
    fn kind(&self) -> EntityKind {
        match self.sprite() {
            "hero" => EntityKind::Player,
            "monster" | "boss" | "guard" => EntityKind::Monster,
            "npc" | "quest_giver" | "vendor" | "ally" => EntityKind::Npc,
            "gate" | "wall" | "bridge" => EntityKind::Structure,
            "mount" | "gem" | "relic" | "scale" | "arrow_stack" | "deed" | "potion" | "blade"
            | "key" | "shield" | "wand" => EntityKind::Item,
            _ => EntityKind::Prop,
        }
    }

    /// Small set of neutral trait tags the client can key off of.
    fn traits(&self) -> Vec<&'static str> {
        let mut tags = vec![self.kind().as_str()];
        if self.kind() == EntityKind::Monster {
            tags.push("attackable");
        }
        if self.has_shield() {
            tags.push("shield");
        }
        tags
    }

    /// True when a monster's label advertises a protective shield.
    fn has_shield(&self) -> bool {
        self.kind() == EntityKind::Monster && self.label().to_ascii_lowercase().contains("shield")
    }

    /// Shield durability parsed from labels like `shield 150`, if present.
    fn shield_hp(&self) -> Option<i64> {
        if !self.has_shield() {
            return None;
        }
        parse_label_number_after(self.label(), "shield ")
            .or_else(|| parse_label_number_after(self.label(), "shield:"))
    }

    /// Maximum shield durability. Scene entities describe a starting state, so
    /// this mirrors `shield_hp` unless a scenario overrides it.
    fn max_shield(&self) -> Option<i64> {
        self.shield_hp()
    }

    /// Skill damage against shields parsed from labels like `50 shield damage`.
    fn shield_damage(&self) -> Option<i64> {
        parse_label_number_before(self.label(), "shield damage")
    }

    /// Current hit points parsed from a trailing `(<n> HP)` label annotation,
    /// if present. Only meaningful for combat entities (monsters).
    fn hp(&self) -> Option<i64> {
        if self.kind() != EntityKind::Monster {
            return None;
        }
        parse_label_hp(self.label())
    }

    /// Maximum hit points. Scene entities describe a starting state, so this
    /// mirrors `hp` unless a scenario overrides it.
    fn max_hp(&self) -> Option<i64> {
        self.hp()
    }
}

/// Parse `160` out of a label like `Arena Monster #1 (160 HP)`.
fn parse_label_hp(label: &str) -> Option<i64> {
    let open = label.rfind('(')?;
    let rest = &label[open + 1..];
    let hp_pos = rest.find("HP").or_else(|| rest.find("hp"))?;
    let digits: String = rest[..hp_pos]
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Parse the first integer that appears immediately after `marker`, e.g.
/// `parse_label_number_after("shield 150, ...", "shield ") == Some(150)`.
fn parse_label_number_after(label: &str, marker: &str) -> Option<i64> {
    let lower = label.to_ascii_lowercase();
    let start = lower.find(marker)? + marker.len();
    let digits: String = label[start..]
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Parse the last integer that appears immediately before `marker`, e.g.
/// `parse_label_number_before("... 50 shield damage", "shield damage") == Some(50)`.
fn parse_label_number_before(label: &str, marker: &str) -> Option<i64> {
    let lower = label.to_ascii_lowercase();
    let end = lower.find(marker)?;
    let digits: String = label[..end]
        .chars()
        .rev()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    digits.parse().ok()
}

#[derive(Debug, Clone, Copy)]
pub struct SceneEntity {
    pub sprite: &'static str,
    pub x: i32,
    pub y: i32,
    pub label: &'static str,
}

impl Entity for SceneEntity {
    fn sprite(&self) -> &str {
        self.sprite
    }
    fn label(&self) -> &str {
        self.label
    }
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
}

// Serialize the derived trait/kind/combat metadata alongside the raw placement
// so the browser renderer can display monster HP/shield on any map without the
// scenario having to repeat that state by hand.
impl Serialize for SceneEntity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hp = self.hp();
        let max_hp = self.max_hp();
        let shield = self.has_shield();
        let shield_hp = self.shield_hp();
        let max_shield = self.max_shield();
        let shield_damage = self.shield_damage();
        let mut len = 6; // sprite, x, y, label, type, traits
        if hp.is_some() {
            len += 1;
        }
        if max_hp.is_some() {
            len += 1;
        }
        if shield {
            len += 1;
        }
        if shield_hp.is_some() {
            len += 1;
        }
        if max_shield.is_some() {
            len += 1;
        }
        if shield_damage.is_some() {
            len += 1;
        }
        let mut state = serializer.serialize_struct("SceneEntity", len)?;
        state.serialize_field("sprite", self.sprite)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("label", self.label)?;
        state.serialize_field("type", self.kind().as_str())?;
        state.serialize_field("traits", &self.traits())?;
        if let Some(hp) = hp {
            state.serialize_field("hp", &hp)?;
        }
        if let Some(max_hp) = max_hp {
            state.serialize_field("max_hp", &max_hp)?;
        }
        if shield {
            state.serialize_field("shield", &true)?;
        }
        if let Some(shield_hp) = shield_hp {
            state.serialize_field("shield_hp", &shield_hp)?;
        }
        if let Some(max_shield) = max_shield {
            state.serialize_field("max_shield", &max_shield)?;
        }
        if let Some(shield_damage) = shield_damage {
            state.serialize_field("shield_damage", &shield_damage)?;
        }
        state.end()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BlockedTile {
    pub x: i32,
    pub y: i32,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Scene {
    pub template: &'static str,
    pub entities: &'static [SceneEntity],
    pub blocked_tiles: &'static [BlockedTile],
}

/// A puzzle. Each scenario lives in its own module and can override any game
/// rule (scene layout, walkability, win condition) by implementing this trait.
pub trait Scenario: Sync {
    fn id(&self) -> &'static str;
    /// Maintainer-facing descriptive title (names the exploit). Never sent to players.
    fn title(&self) -> &'static str;
    /// Player-facing neutral title (does not reveal the method).
    fn player_title(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn difficulty(&self) -> &'static str;
    fn objective(&self) -> &'static str;
    /// Post-mortem shown only after the puzzle is solved.
    fn lesson(&self) -> &'static str;
    fn packets(&self) -> &'static [&'static str];
    fn solution_script(&self) -> &'static str;
    fn naive_script(&self) -> &'static str;
    fn scene(&self) -> Scene;
    fn check_win(&self, events: &[ClientEvent]) -> bool;
    /// Server-side notifications the world emits in response to the player's
    /// packets (e.g. a listing being removed, a mail being created). These are
    /// authoritative game-state updates the client renders; the UI must not
    /// synthesize them locally. Default: none.
    fn notifications(&self, _events: &[ClientEvent]) -> Vec<PacketEvent> {
        Vec::new()
    }
}

pub fn all_scenarios() -> &'static [&'static dyn Scenario] {
    SCENARIOS
}

pub fn find_scenario(id: &str) -> Option<&'static dyn Scenario> {
    SCENARIOS
        .iter()
        .copied()
        .find(|scenario| scenario.id() == id)
}

static SCENARIOS: &[&dyn Scenario] = &[
    &scenario_01_first_blood_batch::SCENARIO,
    &scenario_02_arena_fight_while_dead::SCENARIO,
    &scenario_02_target_validation_range::SCENARIO,
    &scenario_04_target_validation_faction::SCENARIO,
    &scenario_05_auction_negative_price::SCENARIO,
    &scenario_06_auction_buyout_race::SCENARIO,
    &scenario_07_auction_cancel_refund_dupe::SCENARIO,
    &scenario_08_dupe_mail_desync::SCENARIO,
    &scenario_09_dupe_trade_window::SCENARIO,
    &scenario_10_dupe_stack_split_negative::SCENARIO,
    &scenario_11_currency_integer_overflow::SCENARIO,
    &scenario_12_toctou_buy_and_use::SCENARIO,
    &scenario_13_rate_limit_timestamp::SCENARIO,
    &scenario_14_rollback_move_teleport::SCENARIO,
    &scenario_15_replay_signed_loot::SCENARIO,
    &scenario_16_cooldown_bypass_batch::SCENARIO,
    &scenario_17_quest_turnin_double::SCENARIO,
    &scenario_18_instanced_loot_ownership::SCENARIO,
    &scenario_19_quest_cancel_restart_farm::SCENARIO,
    &scenario_20_chest_multi_interaction_dupe::SCENARIO,
    &scenario_21_telehacking_position_spoof::SCENARIO,
    &scenario_22_crafting_clientside_materials::SCENARIO,
];
