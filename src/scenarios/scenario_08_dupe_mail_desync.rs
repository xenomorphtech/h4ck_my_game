#[allow(unused_imports)]
use super::rules::{has, same_tick_count};
use super::{BlockedTile, Scenario, Scene, SceneEntity};
#[allow(unused_imports)]
use crate::engine::{field_i64, field_str, ClientEvent};

pub struct ScenarioImpl;

pub static SCENARIO: ScenarioImpl = ScenarioImpl;

const ENTITIES: &[SceneEntity] = &[SceneEntity {
    sprite: "hero",
    x: 1,
    y: 3,
    label: "You",
}];
const BLOCKED_TILES: &[BlockedTile] = &[];

impl Scenario for ScenarioImpl {
    fn id(&self) -> &'static str {
        "08-dupe-mail-desync"
    }
    fn title(&self) -> &'static str {
        "Phantom Attachment: Mail Desync Dupe"
    }
    fn player_title(&self) -> &'static str {
        "Post Office 1"
    }
    fn category(&self) -> &'static str {
        "Post Office"
    }
    fn difficulty(&self) -> &'static str {
        "★★☆"
    }
    fn objective(&self) -> &'static str {
        "End with two Dragon Scales. You start with one."
    }
    fn lesson(&self) -> &'static str {
        "The attach path and cancel path observed different draft states. Batching attach and cancel let the cancel return an attachment while suppressing the later inventory removal. Fix: commit inventory transfers atomically and make cancellation operate only on committed attachments."
    }
    fn packets(&self) -> &'static [&'static str] {
        &[
            "CreateDraft { recipient: Int }",
            "AttachItem { draft: Int, item: Int }",
            "CancelDraft { draft: Int }",
            "ClaimMail { mail: Int }",
        ]
    }
    fn solution_script(&self) -> &'static str {
        "send CreateDraft { recipient: 0 }\nbatch {\n  send AttachItem { draft: 1, item: 1001 }\n  send CancelDraft { draft: 1 }\n}\n"
    }
    fn naive_script(&self) -> &'static str {
        "send CreateDraft { recipient: 0 }\nsend AttachItem { draft: 1, item: 1001 }\nsleep 50\nsend CancelDraft { draft: 1 }\n"
    }
    fn scene(&self) -> Scene {
        Scene {
            template: "postoffice",
            entities: ENTITIES,
            blocked_tiles: BLOCKED_TILES,
        }
    }
    fn check_win(&self, events: &[ClientEvent]) -> bool {
        has(events, "CreateDraft", "recipient", 0)
            && events.iter().any(|a| {
                a.name == "AttachItem"
                    && events.iter().any(|c| {
                        c.t == a.t
                            && c.name == "CancelDraft"
                            && field_i64(c, "draft") == field_i64(a, "draft")
                    })
            })
    }
}
