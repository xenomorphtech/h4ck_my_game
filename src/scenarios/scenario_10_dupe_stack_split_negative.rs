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
        sprite: "chest",
        x: 4,
        y: 3,
        label: "Bag",
    },
    SceneEntity {
        sprite: "chest",
        x: 5,
        y: 2,
        label: "Bag: slot 0 Arrows, slot 1 empty",
    },
];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "10-dupe-stack-split-negative"
    }
    fn title(&self) -> &'static str {
        "Negative Split: Stack Underflow"
    }
    fn player_title(&self) -> &'static str {
        "Inventory 1"
    }
    fn category(&self) -> &'static str {
        "Inventory"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "Create extra arrows from a single stack."
    }
    fn lesson(&self) -> &'static str {
        "A negative split count moved stack math in the wrong direction and created extra inventory value."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "SplitStack { slot: Int, count: Int }",
            "MergeStack { from_slot: Int, to_slot: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send SplitStack { slot: 0, count: -89 }\nsend MergeStack { from_slot: 1, to_slot: 0 }\n"
    }
    fn naive_script(&self) -> &'static str {
        "send SplitStack { slot: 0, count: 9 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "inventory",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        events
            .iter()
            .any(|x| x.name == "SplitStack" && field_i64(x, "count").is_some_and(|v| v < 0))
            && has(events, "MergeStack", "from_slot", 1)
            && has(events, "MergeStack", "to_slot", 0)
    }
}
