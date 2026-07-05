use crate::engine::ClientEvent;
use serde::Serialize;

pub mod rules;
mod scenario_01_first_blood_batch;
mod scenario_02_arena_fight_while_dead;
mod scenario_02_target_validation_range;
mod scenario_03_target_validation_dead;
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

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SceneEntity {
    pub sprite: &'static str,
    pub x: i32,
    pub y: i32,
    pub label: &'static str,
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
    &scenario_03_target_validation_dead::SCENARIO,
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
