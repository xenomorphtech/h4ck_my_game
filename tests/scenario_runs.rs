use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use packet_hacker::{all_scenarios, app_with_store, run_script, Outcome, Store};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

const README_SCENARIO_IDS: [&str; 23] = [
    "01-first-blood-batch",
    "02-arena-fight-while-dead",
    "02-target-validation-range",
    "03-target-validation-dead",
    "04-target-validation-faction",
    "05-auction-negative-price",
    "06-auction-buyout-race",
    "07-auction-cancel-refund-dupe",
    "08-dupe-mail-desync",
    "09-dupe-trade-window",
    "10-dupe-stack-split-negative",
    "11-currency-integer-overflow",
    "12-toctou-buy-and-use",
    "13-rate-limit-timestamp",
    "14-rollback-move-teleport",
    "15-replay-signed-loot",
    "16-cooldown-bypass-batch",
    "17-quest-turnin-double",
    "18-instanced-loot-ownership",
    "19-quest-cancel-restart-farm",
    "20-chest-multi-interaction-dupe",
    "21-telehacking-position-spoof",
    "22-crafting-clientside-materials",
];

#[test]
fn scenario_registry_matches_readme_index() {
    let actual = all_scenarios()
        .iter()
        .map(|scenario| scenario.id())
        .collect::<BTreeSet<_>>();
    let expected = README_SCENARIO_IDS.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn all_documented_solution_scripts_win() {
    for scenario in all_scenarios() {
        let result = run_script(scenario.id(), scenario.solution_script());
        assert!(
            result.ok,
            "{} solution returned error: {:?}",
            scenario.id(),
            result.error
        );
        assert_eq!(result.outcome, Outcome::Win, "{} should win", scenario.id());
    }
}

/// One explicit test per scenario: its documented `solution_script` must win,
/// and its documented `naive_script` must not. A failure names the exact
/// scenario so regressions are easy to locate.
macro_rules! scenario_solution_tests {
    ($($name:ident => $id:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                let scenario = all_scenarios()
                    .iter()
                    .find(|scenario| scenario.id() == $id)
                    .unwrap_or_else(|| panic!("scenario {} is registered", $id));

                let solved = run_script(scenario.id(), scenario.solution_script());
                assert!(
                    solved.ok,
                    "{} solution returned error: {:?}",
                    $id,
                    solved.error
                );
                assert_eq!(
                    solved.outcome,
                    Outcome::Win,
                    "{} solution_script should win",
                    $id
                );

                let naive = run_script(scenario.id(), scenario.naive_script());
                assert_ne!(
                    naive.outcome,
                    Outcome::Win,
                    "{} naive_script must not win",
                    $id
                );
            }
        )*
    };
}

scenario_solution_tests! {
    solution_01_first_blood_batch => "01-first-blood-batch",
    solution_02_arena_fight_while_dead => "02-arena-fight-while-dead",
    solution_02_target_validation_range => "02-target-validation-range",
    solution_03_target_validation_dead => "03-target-validation-dead",
    solution_04_target_validation_faction => "04-target-validation-faction",
    solution_05_auction_negative_price => "05-auction-negative-price",
    solution_06_auction_buyout_race => "06-auction-buyout-race",
    solution_07_auction_cancel_refund_dupe => "07-auction-cancel-refund-dupe",
    solution_08_dupe_mail_desync => "08-dupe-mail-desync",
    solution_09_dupe_trade_window => "09-dupe-trade-window",
    solution_10_dupe_stack_split_negative => "10-dupe-stack-split-negative",
    solution_11_currency_integer_overflow => "11-currency-integer-overflow",
    solution_12_toctou_buy_and_use => "12-toctou-buy-and-use",
    solution_13_rate_limit_timestamp => "13-rate-limit-timestamp",
    solution_14_rollback_move_teleport => "14-rollback-move-teleport",
    solution_15_replay_signed_loot => "15-replay-signed-loot",
    solution_16_cooldown_bypass_batch => "16-cooldown-bypass-batch",
    solution_17_quest_turnin_double => "17-quest-turnin-double",
    solution_18_instanced_loot_ownership => "18-instanced-loot-ownership",
    solution_19_quest_cancel_restart_farm => "19-quest-cancel-restart-farm",
    solution_20_chest_multi_interaction_dupe => "20-chest-multi-interaction-dupe",
    solution_21_telehacking_position_spoof => "21-telehacking-position-spoof",
    solution_22_crafting_clientside_materials => "22-crafting-clientside-materials",
}

#[test]
fn api_scenario_ids_are_not_rendered_as_player_descriptions() {
    let app_js = include_str!("../client/app.js");
    assert!(
        !app_js.contains("${scenario.id}</small>")
            && !app_js.contains("selected-id').textContent = scenario.id")
            && !app_js.contains("selected-id').textContent=scenario.id"),
        "stable scenario ids contain maintainer-facing bug names and must not be rendered as player descriptions"
    );
}

#[test]
fn all_documented_naive_scripts_do_not_win() {
    for scenario in all_scenarios() {
        let result = run_script(scenario.id(), scenario.naive_script());
        assert_ne!(
            result.outcome,
            Outcome::Win,
            "{} naive script unexpectedly won",
            scenario.id()
        );
    }
}

#[test]
fn arena_two_requires_a_pre_death_hit_then_a_burst_after_death() {
    let scenario = all_scenarios()
        .iter()
        .find(|scenario| scenario.id() == "02-arena-fight-while-dead")
        .copied()
        .expect("arena 2 scenario exists");

    assert_eq!(scenario.player_title(), "Arena 2");

    // Player-facing objective must state only the visible goal, not the
    // death/action-order mechanism the win check actually enforces.
    let objective = scenario.objective().to_lowercase();
    assert!(
        !objective.contains("dead") && !objective.contains("death"),
        "arena 2 objective leaks the death/action-order mechanism: {:?}",
        scenario.objective()
    );

    // The documented solution (one early hit, then a post-death burst) wins.
    assert_eq!(
        run_script(scenario.id(), scenario.solution_script()).outcome,
        Outcome::Win
    );
    // The naive script (no post-death burst) does not.
    assert_ne!(
        run_script(scenario.id(), scenario.naive_script()).outcome,
        Outcome::Win
    );

    // Killing the monster in the wrong order must not win: a single pre-death
    // hit with a later single hit is rejected because the win check enforces a
    // specific action order, not merely that the monster took damage.
    let wrong_order = "send Attack { target: 1 }\nsleep 300\nsend Attack { target: 1 }\n";
    assert_ne!(
        run_script(scenario.id(), wrong_order).outcome,
        Outcome::Win,
        "arena 2 must reject a solution that ignores the required action order"
    );
}

#[test]
fn audited_scenarios_have_visible_affordances_for_required_targets() {
    let scenarios = all_scenarios();

    let gatehouse = scenarios
        .iter()
        .find(|scenario| scenario.id() == "02-target-validation-range")
        .expect("gatehouse scenario exists")
        .scene();
    assert!(gatehouse
        .entities
        .iter()
        .any(|entity| entity.label.contains("Lever #7")));
    assert!(gatehouse
        .blocked_tiles
        .iter()
        .any(|tile| tile.x == 3 && tile.y == 2 && tile.reason == "wall"));
    assert!(gatehouse
        .blocked_tiles
        .iter()
        .any(|tile| tile.x == 3 && tile.y == 4 && tile.reason == "wall"));

    let ruins = scenarios
        .iter()
        .find(|scenario| scenario.id() == "14-rollback-move-teleport")
        .expect("ruins scenario exists")
        .scene();
    assert!(
        ruins
            .blocked_tiles
            .iter()
            .filter(|tile| tile.reason == "wall")
            .count()
            >= 3
    );
    assert!(ruins
        .entities
        .iter()
        .any(|entity| entity.label.contains("Relic #77")));

    let bridge = scenarios
        .iter()
        .find(|scenario| scenario.id() == "21-telehacking-position-spoof")
        .expect("bridge scenario exists")
        .scene();
    assert!(bridge
        .entities
        .iter()
        .any(|entity| entity.label.contains("Shrine #91")));
}

#[test]
fn audited_scenario_mechanics_match_visible_setup() {
    assert_eq!(
        run_script(
            "06-auction-buyout-race",
            "send Buyout { listing: 21, quantity: 2 }\n"
        )
        .outcome,
        Outcome::Lose,
        "buying quantity 2 in one packet must not satisfy the two-buyer race"
    );

    let repeated_nonzero_timestamp =
        "batch {\n  for i in 1..11 {\n    send Zap { target: 1, client_time_ms: 123 }\n  }\n}\n";
    assert_eq!(
        run_script("13-rate-limit-timestamp", repeated_nonzero_timestamp).outcome,
        Outcome::Win,
        "crystal should accept any repeated identical client timestamp, not only magic zero"
    );

    let increasing_timestamps =
        "batch {\n  for i in 1..11 {\n    send Zap { target: 1, client_time_ms: i }\n  }\n}\n";
    assert_eq!(
        run_script("13-rate-limit-timestamp", increasing_timestamps).outcome,
        Outcome::Lose,
        "distinct client timestamps must not overload the crystal"
    );

    assert_eq!(
        run_script(
            "20-chest-multi-interaction-dupe",
            "send OpenChest { chest: 81 }\nsleep 50\nsend OpenChest { chest: 81 }\nsleep 50\nsend OpenChest { chest: 81 }\n"
        )
        .outcome,
        Outcome::Lose,
        "three sequential chest opens should fail; the missing ingredient is timing"
    );

    assert_eq!(
        run_script(
            "21-telehacking-position-spoof",
            "send Interact { target: 91, client_x: 7, client_y: 3 }\n"
        )
        .outcome,
        Outcome::Win,
        "bridge coordinates should match the visible shrine tile"
    );
    assert_eq!(
        run_script(
            "21-telehacking-position-spoof",
            "send Interact { target: 91, client_x: 7, client_y: 0 }\n"
        )
        .outcome,
        Outcome::Lose,
        "bridge interaction should validate the visible shrine y coordinate too"
    );

    assert_eq!(
        run_script(
            "22-crafting-clientside-materials",
            "send CraftItem { recipe: 101, material_count: 2 }\n"
        )
        .outcome,
        Outcome::Win,
        "any under-declared material count should demonstrate the same crafting flaw"
    );
}

#[test]
fn client_combat_feedback_does_not_explain_arena_two_packet_acceptance() {
    let combat_js = include_str!("../client/combat.js");
    assert!(
        !combat_js.contains("accepts attack packets")
            && !combat_js.contains("still accepts attack")
            && !combat_js.contains("dead, but"),
        "Arena 2 death feedback is player-facing and must not explain the packet acceptance rule"
    );
}

#[test]
fn unknown_scenario_returns_structured_error() {
    let result = run_script("missing-scenario", "send Attack { target: 1 }");

    assert!(!result.ok);
    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.scenario_id, "missing-scenario");
    assert!(result.events.is_empty());
    assert!(result
        .state
        .as_object()
        .is_some_and(|state| state.is_empty()));
    assert!(result
        .error
        .as_deref()
        .unwrap_or("")
        .contains("unknown scenario"));
}

#[test]
fn parse_errors_return_structured_error() {
    let result = run_script("01-first-blood-batch", "send Attack { target: }");

    assert!(!result.ok);
    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.scenario_id, "01-first-blood-batch");
    assert!(result.events.is_empty());
    assert!(result
        .error
        .as_deref()
        .unwrap_or("")
        .contains("parse error"));
}

#[tokio::test]
async fn api_scenarios_lists_all_documented_ids() {
    let payload = api_scenarios_payload().await;
    let scenarios = payload.as_array().expect("/api/scenarios returns an array");
    let actual = scenarios
        .iter()
        .map(|scenario| scenario["id"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let expected = README_SCENARIO_IDS.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn api_scenarios_are_player_safe_and_visual() {
    let payload = api_scenarios_payload().await;
    let scenarios = payload.as_array().expect("/api/scenarios returns an array");
    let crafting = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "22-crafting-clientside-materials")
        .expect("crafting scenario exists");

    assert_eq!(crafting["title"], "Crafting 1");
    assert_eq!(
        crafting["example_script"],
        "send CraftItem { recipe: 101, material_count: 4 }\n"
    );
    assert!(
        crafting.get("solution_script").is_none(),
        "player API must not leak solutions"
    );
    assert!(
        crafting.get("naive_script").is_none(),
        "player API should call this example_script"
    );
    assert!(
        crafting.get("lesson").is_none(),
        "lesson is revealed only after solving"
    );
    assert_eq!(crafting["scene"]["template"], "workshop");
    assert!(crafting["scene"]["entities"]
        .as_array()
        .is_some_and(|entities| !entities.is_empty()));

    let inventory = crafting["inventory"]
        .as_array()
        .expect("player API exposes an inventory array");
    assert!(inventory.iter().any(|item| {
        item["name"] == "Dragon Scale" && item["quantity"] == 0 && item["sprite"] == "scale"
    }));

    let arena_two = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "02-arena-fight-while-dead")
        .expect("arena 2 is visible to players");
    assert_eq!(arena_two["title"], "Arena 2");
    assert_eq!(arena_two["scene"]["template"], "arena");
    assert_eq!(arena_two["objective"], "Kill the monster.");
}

#[tokio::test]
async fn api_scenarios_expose_walkability_for_blocked_maps() {
    let payload = api_scenarios_payload().await;
    let scenarios = payload.as_array().expect("/api/scenarios returns an array");
    let gatehouse = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "02-target-validation-range")
        .expect("gatehouse scenario exists");
    let bridge = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "21-telehacking-position-spoof")
        .expect("bridge bypass scenario exists");

    assert!(gatehouse["scene"]["blocked_tiles"]
        .as_array()
        .is_some_and(|tiles| tiles.iter().any(|tile| tile["x"] == 3 && tile["y"] == 3)));
    assert!(bridge["scene"]["blocked_tiles"]
        .as_array()
        .is_some_and(|tiles| tiles.iter().any(|tile| tile["reason"] == "chasm")));
}

#[test]
fn lessons_are_hidden_until_the_puzzle_is_solved() {
    let scenario = all_scenarios()
        .iter()
        .find(|scenario| scenario.id() == "22-crafting-clientside-materials")
        .unwrap();

    let losing = run_script(scenario.id(), scenario.naive_script());
    assert_eq!(losing.outcome, Outcome::Lose);
    assert!(losing.state.get("lesson").is_none());

    let winning = run_script(scenario.id(), scenario.solution_script());
    assert_eq!(winning.outcome, Outcome::Win);
    assert!(winning.state["lesson"]
        .as_str()
        .is_some_and(|lesson| lesson.contains("server-authoritative")));
}

#[test]
fn store_keeps_completed_puzzles_per_user() {
    let store = Store::memory().unwrap();

    store.ensure_user("alice").unwrap();
    store.ensure_user("bob").unwrap();
    store
        .mark_completed("alice", "01-first-blood-batch")
        .unwrap();
    store
        .mark_completed("alice", "22-crafting-clientside-materials")
        .unwrap();
    store
        .mark_completed("alice", "01-first-blood-batch")
        .unwrap();

    assert_eq!(
        store.completed_ids("alice").unwrap(),
        vec![
            "01-first-blood-batch".to_string(),
            "22-crafting-clientside-materials".to_string()
        ]
    );
    assert!(store.completed_ids("bob").unwrap().is_empty());
}

#[tokio::test]
async fn progress_api_sets_user_cookie_and_returns_completed_ids() {
    let store = Store::memory().unwrap();
    store.ensure_user("existing-user").unwrap();
    store
        .mark_completed("existing-user", "01-first-blood-batch")
        .unwrap();

    let app = app_with_store(store);
    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/progress")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first.status(), StatusCode::OK);
    let set_cookie = first
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(set_cookie.starts_with("ph_uid="));
    assert!(set_cookie.contains("Path=/"));
    let first_payload: Value =
        serde_json::from_slice(&to_bytes(first.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(first_payload["completed"].as_array().unwrap().len(), 0);

    let second = app
        .oneshot(
            Request::builder()
                .uri("/api/progress")
                .header(header::COOKIE, "ph_uid=existing-user")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second_payload: Value =
        serde_json::from_slice(&to_bytes(second.into_body(), usize::MAX).await.unwrap()).unwrap();

    assert_eq!(second_payload["user_id"], "existing-user");
    assert_eq!(second_payload["completed"], json!(["01-first-blood-batch"]));
}

async fn api_scenarios_payload() -> Value {
    let response = app_with_store(Store::memory().unwrap())
        .oneshot(
            Request::builder()
                .uri("/api/scenarios")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn websocket_accepts_run_script_and_returns_run_result() {
    let store = Store::memory().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app_with_store(store)).await.unwrap();
    });

    let (mut socket, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();
    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "01-first-blood-batch",
                "script": "batch {\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n}\n"
            })
            .to_string(),
        ))
        .await
        .unwrap();

    let message = socket.next().await.unwrap().unwrap();
    let text = message.into_text().unwrap();
    let payload: Value = serde_json::from_str(&text).unwrap();

    assert_eq!(payload["type"], "run_result");
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["scenario_id"], "01-first-blood-batch");
    assert_eq!(payload["outcome"], "win");
    assert!(payload["events"]
        .as_array()
        .is_some_and(|events| !events.is_empty()));

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn websocket_records_completed_puzzle_for_cookie_user() {
    let store = Store::memory().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_store = store.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, app_with_store(server_store))
            .await
            .unwrap();
    });

    let mut request = format!("ws://{addr}/ws").into_client_request().unwrap();
    request
        .headers_mut()
        .insert("Cookie", "ph_uid=socket-user".parse().unwrap());
    let (mut socket, _) = connect_async(request).await.unwrap();
    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "01-first-blood-batch",
                "script": "batch {\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n  send Attack { target: 1 }\n}\n"
            })
            .to_string(),
        ))
        .await
        .unwrap();

    let message = socket.next().await.unwrap().unwrap();
    let payload: Value = serde_json::from_str(&message.into_text().unwrap()).unwrap();

    assert_eq!(payload["outcome"], "win");
    assert_eq!(
        store.completed_ids("socket-user").unwrap(),
        vec!["01-first-blood-batch".to_string()]
    );

    server.abort();
}
