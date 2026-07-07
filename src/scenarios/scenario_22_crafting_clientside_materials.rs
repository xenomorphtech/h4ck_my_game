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
        sprite: "anvil",
        x: 4,
        y: 3,
        label: "Anvil: recipe #101 needs 4 scales",
    },
    SceneEntity {
        sprite: "blade",
        x: 6,
        y: 2,
        label: "Dragon Blade display",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "22-crafting-clientside-materials"
    }
    fn title(&self) -> &'static str {
        "Recipe Is Just a Suggestion: Client-Side Crafting Materials"
    }
    fn player_title(&self) -> &'static str {
        "Crafting 1"
    }
    fn category(&self) -> &'static str {
        "Workshop"
    }
    fn difficulty(&self) -> &'static str {
        "★★★"
    }
    fn objective(&self) -> &'static str {
        "Forge the Dragon Blade at the anvil."
    }
    fn lesson(&self) -> &'static str {
        "The craft accepted client-declared material counts, so the recipe completed without spending real materials."
    }
    fn packets(&self) -> &'static [&'static str] {
        &["CraftItem { recipe: Int, material_count: Int }"]
    }
    fn solution_script(&self) -> &'static str {
        "send CraftItem { recipe: 101, material_count: 0 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send CraftItem { recipe: 101, material_count: 4 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "workshop",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events.iter().any(|x| {
            x.name == "CraftItem"
                && field_i64(x, "recipe") == Some(101)
                && field_i64(x, "material_count").is_some_and(|count| count < 4)
        })
    }
}
