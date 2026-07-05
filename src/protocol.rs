use crate::scenarios::{Scenario, Scene};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Win,
    Lose,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEvent {
    pub t: u64,
    pub kind: String,
    pub name: String,
    pub fields: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    #[serde(rename = "type")]
    pub message_type: String,
    pub ok: bool,
    pub scenario_id: String,
    pub outcome: Outcome,
    pub time_ms: u64,
    pub state: Value,
    pub events: Vec<PacketEvent>,
    pub error: Option<String>,
}

impl RunResult {
    pub fn error(scenario_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            message_type: "run_result".to_string(),
            ok: false,
            scenario_id: scenario_id.into(),
            outcome: Outcome::Error,
            time_ms: 0,
            state: json!({}),
            events: vec![],
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunScriptRequest {
    #[serde(rename = "type")]
    pub message_type: String,
    pub scenario_id: String,
    pub script: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressResponse {
    pub user_id: String,
    pub completed: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct InventoryItem {
    pub name: &'static str,
    pub sprite: &'static str,
    pub quantity: i32,
    pub slot: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSummary {
    pub id: &'static str,
    /// Player-facing neutral title; the exploit-naming `title` is intentionally omitted.
    pub title: &'static str,
    pub category: &'static str,
    pub difficulty: &'static str,
    pub objective: &'static str,
    pub packets: &'static [&'static str],
    /// The naive starting script the player edits. Never the solution.
    pub example_script: &'static str,
    pub scene: Scene,
    /// Initial player inventory displayed by the reusable inventory UI component.
    pub inventory: &'static [InventoryItem],
}

impl From<&dyn Scenario> for ScenarioSummary {
    fn from(scenario: &dyn Scenario) -> Self {
        Self {
            id: scenario.id(),
            title: scenario.player_title(),
            category: scenario.category(),
            difficulty: scenario.difficulty(),
            objective: scenario.objective(),
            packets: scenario.packets(),
            example_script: scenario.naive_script(),
            scene: scenario.scene(),
            inventory: inventory_for(scenario.id()),
        }
    }
}

const EMPTY_INVENTORY: &[InventoryItem] = &[];
const ARENA_1_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Training Sword",
        sprite: "blade",
        quantity: 1,
        slot: "weapon",
    },
    InventoryItem {
        name: "Small Potion",
        sprite: "potion",
        quantity: 1,
        slot: "bag",
    },
];
const GATEHOUSE_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Lever #7",
    sprite: "key",
    quantity: 0,
    slot: "target",
}];
const CRYPT_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Training Sword",
    sprite: "blade",
    quantity: 1,
    slot: "weapon",
}];
const SIEGE_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Allied Cannon #2",
    sprite: "cannon",
    quantity: 1,
    slot: "siege unit",
}];
const ARENA_SKILL_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Skill Crystal #10",
    sprite: "wand",
    quantity: 1,
    slot: "skill",
}];
const MARKET_MOUNT_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Gold",
        sprite: "currency",
        quantity: 100,
        slot: "wallet",
    },
    InventoryItem {
        name: "Phoenix Mount",
        sprite: "mount",
        quantity: 0,
        slot: "goal",
    },
];
const MARKET_GEM_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Gold",
        sprite: "currency",
        quantity: 1000,
        slot: "wallet",
    },
    InventoryItem {
        name: "Gem",
        sprite: "gem",
        quantity: 0,
        slot: "goal",
    },
];
const MARKET_REFUND_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Listed Sword",
        sprite: "blade",
        quantity: 0,
        slot: "auction",
    },
    InventoryItem {
        name: "Gold",
        sprite: "currency",
        quantity: 0,
        slot: "wallet",
    },
];
const MAIL_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Dragon Scale",
        sprite: "scale",
        quantity: 1,
        slot: "bag",
    },
    InventoryItem {
        name: "Mail Draft",
        sprite: "mailbox",
        quantity: 0,
        slot: "remote",
    },
];
const TRADE_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Shield",
        sprite: "shield",
        quantity: 1,
        slot: "offer",
    },
    InventoryItem {
        name: "Potion",
        sprite: "potion",
        quantity: 0,
        slot: "receive",
    },
];
const ARROWS_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Arrows",
        sprite: "arrow_stack",
        quantity: 10,
        slot: "slot 0",
    },
    InventoryItem {
        name: "Empty Slot",
        sprite: "chest",
        quantity: 0,
        slot: "slot 1",
    },
];
const TREASURY_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Gold",
        sprite: "currency",
        quantity: 100,
        slot: "wallet",
    },
    InventoryItem {
        name: "Castle Deed",
        sprite: "deed",
        quantity: 0,
        slot: "goal",
    },
];
const APOTHECARY_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Gold",
        sprite: "currency",
        quantity: 1,
        slot: "wallet",
    },
    InventoryItem {
        name: "Antidote",
        sprite: "potion",
        quantity: 0,
        slot: "goal",
    },
];
const CRYSTAL_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Lightning Wand",
    sprite: "wand",
    quantity: 1,
    slot: "weapon",
}];
const VAULT_RELIC_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Relic",
    sprite: "relic",
    quantity: 0,
    slot: "goal",
}];
const GUILD_REWARD_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Quest Item",
        sprite: "deed",
        quantity: 1,
        slot: "bag",
    },
    InventoryItem {
        name: "Reward Chest",
        sprite: "chest",
        quantity: 0,
        slot: "goal",
    },
];
const RAID_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Ally's Drop",
    sprite: "relic",
    quantity: 0,
    slot: "reserved",
}];
const PROVISION_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Provision Kit",
    sprite: "potion",
    quantity: 0,
    slot: "goal",
}];
const CHEST_BUNDLE_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Reward Bundle",
    sprite: "chest",
    quantity: 0,
    slot: "goal",
}];
const SHRINE_INVENTORY: &[InventoryItem] = &[InventoryItem {
    name: "Shrine Charge",
    sprite: "shrine",
    quantity: 0,
    slot: "goal",
}];
const CRAFTING_INVENTORY: &[InventoryItem] = &[
    InventoryItem {
        name: "Dragon Scale",
        sprite: "scale",
        quantity: 0,
        slot: "materials (need 4)",
    },
    InventoryItem {
        name: "Dragon Blade",
        sprite: "blade",
        quantity: 0,
        slot: "goal",
    },
];

fn inventory_for(id: &str) -> &'static [InventoryItem] {
    match id {
        "01-first-blood-batch" | "02-arena-fight-while-dead" => ARENA_1_INVENTORY,
        "02-target-validation-range" => GATEHOUSE_INVENTORY,
        "03-target-validation-dead" => CRYPT_INVENTORY,
        "04-target-validation-faction" => SIEGE_INVENTORY,
        "16-cooldown-bypass-batch" => ARENA_SKILL_INVENTORY,
        "05-auction-negative-price" => MARKET_MOUNT_INVENTORY,
        "06-auction-buyout-race" => MARKET_GEM_INVENTORY,
        "07-auction-cancel-refund-dupe" => MARKET_REFUND_INVENTORY,
        "08-dupe-mail-desync" => MAIL_INVENTORY,
        "09-dupe-trade-window" => TRADE_INVENTORY,
        "10-dupe-stack-split-negative" => ARROWS_INVENTORY,
        "11-currency-integer-overflow" => TREASURY_INVENTORY,
        "12-toctou-buy-and-use" => APOTHECARY_INVENTORY,
        "13-rate-limit-timestamp" | "14-rollback-move-teleport" => CRYSTAL_INVENTORY,
        "15-replay-signed-loot" => VAULT_RELIC_INVENTORY,
        "17-quest-turnin-double" => GUILD_REWARD_INVENTORY,
        "18-instanced-loot-ownership" => RAID_INVENTORY,
        "19-quest-cancel-restart-farm" => PROVISION_INVENTORY,
        "20-chest-multi-interaction-dupe" => CHEST_BUNDLE_INVENTORY,
        "21-telehacking-position-spoof" => SHRINE_INVENTORY,
        "22-crafting-clientside-materials" => CRAFTING_INVENTORY,
        _ => EMPTY_INVENTORY,
    }
}
