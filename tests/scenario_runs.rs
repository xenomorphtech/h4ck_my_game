use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use packet_hacker::{all_scenarios, app_with_store, run_script, Outcome, Store};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

const README_SCENARIO_IDS: [&str; 22] = [
    "01-first-blood-batch",
    "02-arena-fight-while-dead",
    "02-target-validation-range",
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
    assert!(
        app_js.contains("state: visibleRunState(result.state)")
            && app_js.contains("delete out.scenario_id")
            && app_js.contains("delete out.internal_id")
            && app_js.contains("delete out.bug_slug"),
        "run result state shown in the Result tab must strip internal metadata keys"
    );
}

#[test]
fn run_results_use_player_titles_for_visible_scenario_fields() {
    let result = run_script(
        "07-auction-cancel-refund-dupe",
        "send ClaimMailbox { mail: 1 }\n",
    );
    assert_eq!(
        result.scenario_id, "07-auction-cancel-refund-dupe",
        "the stable id remains internal metadata for routing and progress"
    );

    let state = result.state.as_object().expect("run state is an object");
    assert_eq!(state["scenario"], "market-3");
    assert_eq!(state["title"], "market-3");
    assert!(
        !serde_json::to_string(&result.state)
            .unwrap()
            .contains("auction-cancel-refund-dupe"),
        "visible result state must not expose the maintainer bug slug"
    );
    assert!(
        result.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "ObjectiveFailed"
                && event.fields["scenario"] == "market-3"
        }),
        "visible objective events should use the player-facing puzzle title"
    );
    assert!(
        !serde_json::to_string(&result.events)
            .unwrap()
            .contains("auction-cancel-refund-dupe"),
        "visible event log fields must not expose the maintainer bug slug"
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
fn market_three_cancel_refund_is_not_tied_to_a_magic_timestamp() {
    let scenario_id = "07-auction-cancel-refund-dupe";

    let manual_cancel_then_claim = run_script(
        scenario_id,
        "send CancelListing { listing: 32 }\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n",
    );
    assert_eq!(
        manual_cancel_then_claim.outcome,
        Outcome::Win,
        "canceling the sold sword listing and claiming both resulting rewards should not require an exact timestamp"
    );

    let delayed_cancel_then_claim = run_script(
        scenario_id,
        "sleep 125\nsend CancelListing { listing: 32 }\nsleep 75\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n",
    );
    assert_eq!(
        delayed_cancel_then_claim.outcome,
        Outcome::Win,
        "manual action timing should not be the win condition"
    );

    let cancel_visible_non_objective_listing = run_script(
        scenario_id,
        "send CancelListing { listing: 31 }\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n",
    );
    assert_ne!(
        cancel_visible_non_objective_listing.outcome,
        Outcome::Win,
        "the visible UI-cancelable listing is not the objective sword and must not solve the puzzle"
    );

    let claim_return_without_cancel = run_script(
        scenario_id,
        "send ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n",
    );
    assert_ne!(
        claim_return_without_cancel.outcome,
        Outcome::Win,
        "mail #2 is created by the cancel path and must not be claimable by itself"
    );

    let claim_return_before_cancel = run_script(
        scenario_id,
        "send ClaimMailbox { mail: 2 }\nsend CancelListing { listing: 31 }\nsend ClaimMailbox { mail: 1 }\n",
    );
    assert_ne!(
        claim_return_before_cancel.outcome,
        Outcome::Win,
        "mail #2 cannot be claimed before the cancel action creates it"
    );

    let decoy_then_sword_claiming_decoy_mail = run_script(
        scenario_id,
        "send CancelListing { listing: 31 }\nsend CancelListing { listing: 32 }\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 2 }\n",
    );
    assert_ne!(
        decoy_then_sword_claiming_decoy_mail.outcome,
        Outcome::Win,
        "if the Copper Charm creates mail #2, claiming #2 is not claiming the returned sword"
    );

    let decoy_then_sword_claiming_sword_mail = run_script(
        scenario_id,
        "send CancelListing { listing: 31 }\nsend CancelListing { listing: 32 }\nsend ClaimMailbox { mail: 1 }\nsend ClaimMailbox { mail: 3 }\n",
    );
    assert_eq!(
        decoy_then_sword_claiming_sword_mail.outcome,
        Outcome::Win,
        "mail ids should be allocated in creation order, so the sword return is #3 after the decoy cancel"
    );
}

#[test]
fn market_and_mail_state_is_not_duplicated_as_canvas_units() {
    // The market/mail panels now own auction and mailbox state, so those props
    // must not also be rendered as scene "units" on the map.
    for scenario in all_scenarios() {
        let scene = scenario.scene();
        for entity in scene.entities {
            assert!(
                entity.sprite != "auction" && entity.sprite != "mailbox",
                "{} renders {:?} as a canvas unit; auction/mailbox state belongs in the market/mail panels",
                scenario.id(),
                entity.sprite
            );
        }
    }
}

#[test]
fn arena_one_emits_authoritative_combat_packets() {
    let scenario = all_scenarios()
        .iter()
        .find(|scenario| scenario.id() == "01-first-blood-batch")
        .copied()
        .expect("arena 1 scenario exists");

    let solved = run_script(scenario.id(), scenario.solution_script());
    assert_eq!(solved.outcome, Outcome::Win);
    assert_eq!(
        solved
            .events
            .iter()
            .filter(|event| {
                event.kind == "server"
                    && event.name == "Damage"
                    && event.fields["source"] == json!(0)
                    && event.fields["target"] == json!(1)
                    && event.fields["amount"] == json!(40)
            })
            .count(),
        3,
        "the batched kill should render three server monster-damage packets"
    );
    assert!(
        solved.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Death"
                && event.t == 0
                && event.fields["target"] == json!(1)
        }),
        "the backend must emit monster death for the Arena 1 solution"
    );
    assert!(
        !solved.events.iter().any(|event| {
            event.kind == "server" && event.name == "Death" && event.fields["target"] == json!(0)
        }),
        "the monster should not retaliate after the same-tick kill"
    );

    let naive = run_script(scenario.id(), scenario.naive_script());
    assert_ne!(naive.outcome, Outcome::Win);
    assert!(
        naive.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Damage"
                && event.t == 250
                && event.fields["source"] == json!(1)
                && event.fields["target"] == json!(0)
                && event.fields["amount"] == json!(999)
        }),
        "the backend must emit the monster's fatal Arena 1 retaliation"
    );
    assert!(
        naive.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Death"
                && event.t == 250
                && event.fields["target"] == json!(0)
        }),
        "the backend must emit player death for the naive Arena 1 path"
    );
    assert!(
        !naive.events.iter().any(|event| {
            event.kind == "server" && event.name == "Death" && event.fields["target"] == json!(1)
        }),
        "the naive Arena 1 path should not render a monster kill"
    );
}

#[test]
fn arena_two_win_condition_is_monster_dead_not_batch_shape() {
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

    // The documented solution should demonstrate that attacks after death are
    // still accepted, without requiring a batch operation.
    assert_eq!(
        run_script(scenario.id(), scenario.solution_script()).outcome,
        Outcome::Win
    );
    assert!(
        !scenario.solution_script().contains("batch"),
        "Arena 2 is about fighting after death, not batch timing"
    );

    // A non-batched sequence of cooldown-spaced attacks should win because the
    // monster is dead, not because of how packets were grouped. The first hit
    // causes a server retaliation/death, but this scenario's bug still accepts
    // later attacks from the dead player.
    let sequential_kill = "send Attack { target: 1 }\nsleep 750\nsend Attack { target: 1 }\nsleep 750\nsend Attack { target: 1 }\nsleep 750\nsend Attack { target: 1 }\n";
    let sequential_result = run_script(scenario.id(), sequential_kill);
    assert_eq!(
        sequential_result.outcome,
        Outcome::Win,
        "four cooldown-spaced 40-damage hits should kill the 160 HP monster"
    );
    assert!(
        sequential_result
            .events
            .iter()
            .any(|event| event.kind == "server"
                && event.name == "Death"
                && event.fields["target"] == json!(0)),
        "the backend must emit the player's death instead of the frontend inventing retaliation"
    );
    assert!(
        sequential_result
            .events
            .iter()
            .any(|event| event.kind == "server"
                && event.name == "Death"
                && event.fields["target"] == json!(1)),
        "the backend must emit monster death when HP reaches zero"
    );

    // Batch/too-fast attacks are not enough: Arena 2 is not a batch puzzle, and
    // its normal attack cooldown is still enforced by the backend simulation.
    let cooldown_violating_batch = "send_batch {\n  Attack { target: 1 }\n  Attack { target: 1 }\n  Attack { target: 1 }\n  Attack { target: 1 }\n}\n";
    let cooldown_result = run_script(scenario.id(), cooldown_violating_batch);
    assert_ne!(
        cooldown_result.outcome,
        Outcome::Win,
        "same-tick attacks must not bypass Arena 2's attack cooldown"
    );
    let cooldown_errors = cooldown_result
        .events
        .iter()
        .filter(|event| {
            event.kind == "server"
                && event.name == "Info"
                && event.fields["level"] == json!("error")
                && event.fields["reason"] == json!("cooldown")
                && event.fields["action"] == json!("Attack")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cooldown_errors.len(),
        3,
        "the server should explain each cooldown-dropped Attack"
    );
    assert!(cooldown_errors.iter().all(|event| {
        event.fields["ready_at_ms"] == json!(750) && event.fields["remaining_ms"] == json!(750)
    }));

    // The naive script still does not win because its post-death attacks arrive
    // during cooldown and leave the monster alive.
    assert_ne!(
        run_script(scenario.id(), scenario.naive_script()).outcome,
        Outcome::Win
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
        "send_batch {\n  for i in 1..11 {\n    Zap { target: 1, client_time_ms: 123 }\n  }\n}\n";
    assert_eq!(
        run_script("13-rate-limit-timestamp", repeated_nonzero_timestamp).outcome,
        Outcome::Win,
        "crystal should accept any repeated identical client timestamp, not only magic zero"
    );

    let increasing_timestamps =
        "send_batch {\n  for i in 1..11 {\n    Zap { target: 1, client_time_ms: i }\n  }\n}\n";
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
fn market_cancel_button_sends_packet_and_applies_server_notifications() {
    let app_js = include_str!("../client/app.js");
    // The cancel button must submit the real packet. Listing removal and returned
    // mail are then driven by server notification events in the run result, not
    // by the click handler mutating market/mail state locally.
    assert!(
        app_js.contains("cancel.onclick = () => sendPacketScript(`send ${listing.cancel_packet}`)")
            && app_js.contains(
                "claim.onclick = () => sendPacketScript(`send ClaimMailbox { mail: ${message.id} }`)"
            )
            && !app_js.contains("function sendSystemPacketScript"),
        "market/mail action buttons should submit their packet directly without appending to the editor"
    );
    assert!(
        app_js.contains("sendScript(`${line}\\n`, `sent packet: ${line}`, actionSessionStarted)")
            && app_js.contains("game.run_script_json(selected.id, script, append)")
            && app_js.contains("game.reset_script_session(selected.id)")
            && app_js.contains("actionSessionStarted = false"),
        "packet-button sends should preserve local wasm world state without mutating the editor"
    );
    assert!(
        app_js.contains("function applyServerNotifications"),
        "client must apply server-side notifications to refresh market/mail UI"
    );
    assert!(
        app_js.contains("function rebaseSystemViews"),
        "each stateless run result should rebuild market/mail/inventory from the scenario baseline before applying server notifications"
    );
    assert!(app_js.contains("ListingRemoved"));
    assert!(app_js.contains("MailCreated"));
    assert!(app_js.contains("MailClaimed"));
    assert!(app_js.contains("InventoryAdded"));
    assert!(app_js.contains("className = 'mail-claim'"));
    assert!(app_js.contains("send ClaimMailbox { mail: ${message.id} }"));
    assert!(
        !app_js
            .contains("cancel.onclick = () => appendScriptLine(`send ${listing.cancel_packet}`)"),
        "cancel must not just append the packet to the editor"
    );
}

#[test]
fn market_cancel_result_contains_server_notifications_for_ui_state() {
    let result = run_script(
        "07-auction-cancel-refund-dupe",
        "send CancelListing { listing: 31 }\n",
    );
    assert_ne!(
        result.outcome,
        Outcome::Win,
        "canceling the visible non-sword listing must not complete the sword objective"
    );
    assert!(result.events.iter().any(|event| {
        event.kind == "server" && event.name == "ListingRemoved" && event.fields["listing"] == 31
    }));
    assert!(result.events.iter().any(|event| {
        event.kind == "server"
            && event.name == "MailCreated"
            && event.fields["mail"] == 2
            && event.fields["attachment"] == "Copper Charm"
            && event.fields["sprite"] == "gem"
            && event.fields["status"] == "unclaimed"
    }));
}

#[test]
fn market_mail_claims_populate_inventory_from_server_notifications() {
    let sale_claim = run_script(
        "07-auction-cancel-refund-dupe",
        "send ClaimMailbox { mail: 1 }\n",
    );
    assert_ne!(
        sale_claim.outcome,
        Outcome::Win,
        "claiming sale proceeds alone is the editable starting script, not the solution"
    );
    assert!(sale_claim.events.iter().any(|event| {
        event.kind == "server"
            && event.name == "MailClaimed"
            && event.fields["mail"] == 1
            && event.fields["status"] == "claimed"
    }));
    assert!(sale_claim.events.iter().any(|event| {
        event.kind == "server"
            && event.name == "InventoryAdded"
            && event.fields["item"] == "Gold"
            && event.fields["sprite"] == "currency"
            && event.fields["quantity"] == 300
            && event.fields["slot"] == "wallet"
    }));

    let decoy_cancel_then_claim = run_script(
        "07-auction-cancel-refund-dupe",
        "send CancelListing { listing: 31 }\nsend ClaimMailbox { mail: 2 }\n",
    );
    assert_ne!(
        decoy_cancel_then_claim.outcome,
        Outcome::Win,
        "canceling and claiming the Copper Charm is not the sword objective"
    );
    assert!(decoy_cancel_then_claim.events.iter().any(|event| {
        event.kind == "server"
            && event.name == "InventoryAdded"
            && event.fields["item"] == "Copper Charm"
            && event.fields["sprite"] == "gem"
            && event.fields["quantity"] == 1
    }));
}

#[test]
fn market_mail_claim_failures_return_server_feedback() {
    let missing_mail = run_script(
        "07-auction-cancel-refund-dupe",
        "send ClaimMailbox { mail: 32 }\n",
    );
    assert_ne!(missing_mail.outcome, Outcome::Win);
    assert!(
        missing_mail.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "MailClaimFailed"
                && event.fields["mail"] == 32
                && event.fields["reason"] == "not_found"
        }),
        "claiming a nonexistent mail id should produce explicit S2C failure feedback"
    );

    let sword_cancel = run_script(
        "07-auction-cancel-refund-dupe",
        "send CancelListing { listing: 32 }\n",
    );
    assert_ne!(sword_cancel.outcome, Outcome::Win);
    assert!(
        sword_cancel.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "MailCreated"
                && event.fields["mail"] == 2
                && event.fields["attachment"] == "Listed Sword"
        }),
        "the raw sold-listing cancel should expose the created return-mail id"
    );
}

#[tokio::test]
async fn arena_three_visible_setup_explains_shield_math_and_retaliation() {
    let payload = api_scenarios_payload().await;
    let scenarios = payload.as_array().expect("/api/scenarios returns an array");
    let arena_three = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "16-cooldown-bypass-batch")
        .expect("arena 3 exists");

    assert!(arena_three["objective"]
        .as_str()
        .is_some_and(|objective| objective.contains("retaliates")));
    let objective = arena_three["objective"].as_str().unwrap_or("");
    assert!(objective.contains("Shield: 150"));
    assert!(objective.contains("50 shield damage"));
    assert!(objective.contains("1000ms cooldown"));
    assert!(objective.contains("500ms"));
    assert!(!objective.contains("batch"));

    let entities = arena_three["scene"]["entities"]
        .as_array()
        .expect("arena 3 entities are an array");
    let boss = entities
        .iter()
        .find(|entity| entity["label"].as_str().unwrap_or("").contains("Boss"))
        .expect("arena 3 has a boss");
    assert_eq!(boss["type"], "monster");
    assert_eq!(boss["shield"], true);
    assert_eq!(boss["hp"], 1);
    assert_eq!(boss["max_hp"], 1);
    assert_eq!(boss["shield_hp"], 150);
    assert_eq!(boss["max_shield"], 150);

    let skill_source = entities
        .iter()
        .find(|entity| {
            entity["label"]
                .as_str()
                .unwrap_or("")
                .contains("PowerStrike")
        })
        .expect("arena 3 has a visible skill source");
    assert_eq!(skill_source["shield_damage"], 50);

    assert!(arena_three["packets"]
        .as_array()
        .is_some_and(|packets| packets
            .iter()
            .any(|packet| packet == "Attack { target: Int }")));
    let skills = arena_three["skills"]["actions"]
        .as_array()
        .expect("arena 3 exposes skill rows");
    assert_eq!(skills.len(), 2);
    let attack = skills
        .iter()
        .find(|skill| skill["name"] == "Attack")
        .expect("Attack is exposed as a skill");
    assert_eq!(attack["cast_packet"], "Attack { target: 1 }");
    assert_eq!(attack["description"], "10 HP damage, 750ms cooldown");
    let power_strike = skills
        .iter()
        .find(|skill| skill["name"] == "PowerStrike")
        .expect("PowerStrike is exposed as a skill");
    assert_eq!(
        power_strike["description"],
        "50 shield damage, 1000ms cooldown"
    );
    assert_eq!(
        power_strike["cast_packet"],
        "CastSkill { skill: 10, target: 1 }"
    );

    assert_ne!(
        run_script(
            "16-cooldown-bypass-batch",
            "send_batch {\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n}\n"
        )
        .outcome,
        Outcome::Win,
        "breaking the shield alone should not defeat a 1 HP boss"
    );
    assert_eq!(
        run_script(
            "16-cooldown-bypass-batch",
            "send_batch {\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  Attack { target: 1 }\n}\n"
        )
        .outcome,
        Outcome::Win,
        "the boss should be defeated only after the shield is broken and Attack lands in the same retaliation window"
    );
    let batched_solution = run_script(
        "16-cooldown-bypass-batch",
        "send_batch {\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  Attack { target: 1 }\n}\n",
    );
    assert!(
        !batched_solution.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Info"
                && event.fields["reason"] == json!("cooldown")
        }),
        "same-frame Arena 3 casts are the intended cooldown bypass and should not be rejected"
    );
    assert_eq!(
        batched_solution
            .events
            .iter()
            .filter(|event| {
                event.kind == "server"
                    && event.name == "Damage"
                    && event.fields["source"] == json!(0)
                    && event.fields["target"] == json!(1)
                    && event.fields["amount"] == json!(50)
            })
            .count(),
        3,
        "Arena 3 should emit one server shield-damage packet per accepted PowerStrike"
    );
    assert!(
        batched_solution.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Damage"
                && event.t == 0
                && event.fields["source"] == json!(0)
                && event.fields["target"] == json!(1)
                && event.fields["amount"] == json!(10)
        }),
        "Arena 3 should emit the finishing Attack damage after the shield breaks"
    );
    assert!(
        batched_solution.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Death"
                && event.t == 0
                && event.fields["target"] == json!(1)
        }),
        "Arena 3 should emit boss death for the batched solution"
    );
    assert!(
        !batched_solution.events.iter().any(|event| {
            event.kind == "server" && event.name == "Death" && event.fields["target"] == json!(0)
        }),
        "Arena 3 should not emit retaliation after the same-frame kill"
    );
    let shield_only = run_script(
        "16-cooldown-bypass-batch",
        "send_batch {\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n}\n",
    );
    assert!(
        shield_only.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Death"
                && event.t == 500
                && event.fields["target"] == json!(0)
        }),
        "Arena 3 should emit the boss retaliation when the shield breaks but the boss lives"
    );
    assert_ne!(
        run_script(
            "16-cooldown-bypass-batch",
            "send CastSkill { skill: 10, target: 1 }\nsleep 500\nsend_batch {\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  CastSkill { skill: 10, target: 1 }\n  Attack { target: 1 }\n}\n"
        )
        .outcome,
        Outcome::Win,
        "cooldown-rejected follow-up bursts must not satisfy Arena 3's win condition"
    );
    let cooldown_feedback = run_script(
        "16-cooldown-bypass-batch",
        "send CastSkill { skill: 10, target: 1 }\nsleep 500\nsend CastSkill { skill: 10, target: 1 }\nsend Attack { target: 1 }\nsleep 300\nsend Attack { target: 1 }\n",
    );
    assert!(
        cooldown_feedback.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Info"
                && event.fields["level"] == json!("error")
                && event.fields["reason"] == json!("cooldown")
                && event.fields["action"] == json!("PowerStrike")
                && event.fields["skill"] == json!(10)
                && event.fields["ready_at_ms"] == json!(1000)
                && event.fields["remaining_ms"] == json!(500)
        }),
        "skill cooldown drops should emit an S2C Info error"
    );
    assert!(
        cooldown_feedback.events.iter().any(|event| {
            event.kind == "server"
                && event.name == "Info"
                && event.fields["level"] == json!("error")
                && event.fields["reason"] == json!("cooldown")
                && event.fields["action"] == json!("Attack")
                && event.fields["ready_at_ms"] == json!(1250)
                && event.fields["remaining_ms"] == json!(450)
        }),
        "attack cooldown drops should emit an S2C Info error"
    );

    let combat_js = include_str!("../client/combat.js");
    assert!(combat_js.contains("'16-cooldown-bypass-batch'"));
    assert!(combat_js.contains("retaliationDelayMs: 500"));
    assert!(combat_js.contains("attackDamage: 10"));
    assert!(combat_js.contains("attackCooldownMs: 750"));
    let app_js = include_str!("../client/app.js");
    assert!(app_js.contains("skill-card"));
    assert!(app_js.contains("appendScriptLine(`send ${skill.cast_packet}`)"));
    let style_css = include_str!("../client/style.css");
    assert!(style_css.contains(".skill-list"));
    assert!(style_css.contains("grid-template-columns: 1fr"));
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
fn frontend_combat_renders_backend_packets_instead_of_inventing_retaliation() {
    let combat_js = include_str!("../client/combat.js");
    let scene_js = include_str!("../client/scene.js");
    let app_js = include_str!("../client/app.js");

    assert!(
        combat_js.contains("packetForAction")
            && app_js.contains("const combatPacket = combat.packetForAction(action)")
            && app_js.contains("sendPacketScript(combatPacket)"),
        "clicking a monster should send the real Attack packet to the Rust engine"
    );
    assert!(
        !combat_js.contains("run the script to resolve combat on the server"),
        "monster clicks should behave like gameplay, not ask the player to manually run the script"
    );
    assert!(
        combat_js.contains("applyServerDamage") && combat_js.contains("applyServerDeath"),
        "combat HUD should update from authoritative Damage/Death packets"
    );
    assert!(
        !combat_js.contains("setTimeout")
            && !combat_js.contains("scheduleRetaliation")
            && !combat_js.contains("retaliationTimer"),
        "frontend must not schedule or invent monster retaliation"
    );
    assert!(
        scene_js.contains("addServerCombatStrike")
            && scene_js.contains("ev.kind === 'server'")
            && scene_js.contains("ev.name === 'Damage'"),
        "scene renderer should draw combat strikes from authoritative Damage packets"
    );
    assert!(
        !scene_js.contains("now + 250")
            && !scene_js.contains("enemy retaliates")
            && !scene_js.contains("retaliation")
            && !scene_js.contains("Player's strike preview"),
        "scene renderer must not draw hardcoded or speculative combat strikes"
    );
}

#[test]
fn frontend_console_groups_script_packets_result_and_events_as_tabs() {
    let html = include_str!("../client/index.html");
    let combat_js = include_str!("../client/combat.js");
    let app_js = include_str!("../client/app.js");
    let style_css = include_str!("../client/style.css");

    assert!(
        html.contains("class=\"console-tabs\"")
            && html.contains("data-tab=\"script-tab\"")
            && html.contains("data-tab=\"syntax-tab\"")
            && html.contains("data-tab=\"packets-tab\"")
            && html.contains("data-tab=\"result-tab\"")
            && html.contains("data-tab=\"events-tab\""),
        "script, syntax, packets, run result, and event log should be grouped as tabs in the packet console"
    );
    assert!(
        html.contains("<div id=\"tab-syntax\"")
            && html.contains("send Packet { field: value }")
            && html.contains("sleep ms")
            && html.contains("send_batch { Packet { field: value } ... }")
            && html.contains("Sends a batch of packets on the same packet frame")
            && html.contains("at(ms) { ... }")
            && html.contains("for i in start..end { ... }")
            && html.contains("await Packet { ... }")
            && html.contains("send Attack { target: 1, power: 40 }")
            && html.contains("for x in 1..8 {")
            && html.contains("Use the Packets tab for the current schema."),
        "syntax tab should document valid script syntax, clarifications, and examples"
    );
    assert!(
        html.contains("<div id=\"tab-packets\"")
            && html.contains("<pre id=\"packets\"></pre>")
            && html.contains("<div id=\"tab-result\"")
            && html.contains("<pre id=\"result\"></pre>")
            && html.contains("<div id=\"tab-events\"")
            && html.contains("<div id=\"events\"></div>"),
        "packets, run result, and event log panes should live in the same dock as the script editor"
    );
    assert!(
        !html.contains("<details class=\"panel\"")
            && !html.contains("<summary>Packets you can send</summary>"),
        "packets/result/events should no longer be separate stage details panels"
    );
    assert!(
        html.contains("Flag captured")
            && html.contains("Exploit accepted.")
            && html.contains("Exploit recap")
            && app_js.contains("? 'flag captured'")
            && combat_js.contains("this.setStatus('flag captured')")
            && combat_js.contains("this.dom.popupTitle.textContent = 'Flag captured'")
            && combat_js.contains("this.killMonster('Exploit accepted.')"),
        "completion UI should use CTF-style exploit language"
    );
    assert!(
        app_js.contains("function activateConsoleTab")
            && app_js.contains("tab.onclick = () => activateConsoleTab(tab.dataset.tab)"),
        "console tabs should be interactive"
    );
    assert!(
        app_js.contains("import initWasm, { WasmGame } from './pkg/packet_hacker.js'")
            && app_js.contains("game = new WasmGame()")
            && app_js.contains("wasmGame.scenarios_json()")
            && app_js.contains("game.run_script_json(selected.id, script, append)")
            && app_js.contains("game.challenge_state_json(completedDatabaseJson())")
            && app_js.contains("COMPLETED_DB_KEY")
            && app_js.contains("localStorage.getItem(COMPLETED_DB_KEY)")
            && app_js.contains("localStorage.setItem(COMPLETED_DB_KEY")
            && !app_js.contains("fetch('/api/scenarios')")
            && !app_js.contains("fetch('/api/progress')")
            && !app_js.contains("new WebSocket")
            && html.contains("<script type=\"module\" src=\"/client/app.js\"></script>")
            && app_js.contains("button.classList.toggle('upcoming', state.upcoming)")
            && app_js.contains("button.disabled = !state.enabled")
            && app_js.contains("Number(!a.state.completed) - Number(!b.state.completed)"),
        "frontend should use local wasm plus localStorage completion state and render disabled upcoming puzzles sorted below completed ones"
    );
    assert!(
        !app_js.contains("activateConsoleTab('result-tab')"),
        "receiving a run result should not steal focus from the current console tab"
    );
    assert!(
        style_css.contains(".console-tabs")
            && style_css.contains(".console-tab.active")
            && style_css.contains(".console-pane")
            && style_css.contains(".syntax-scroll")
            && style_css.contains(".syntax-section")
            && style_css.contains(".scenario.upcoming"),
        "tab dock and syntax reference should have dedicated layout styles"
    );
}

#[test]
fn wasm_api_exposes_scenarios_runner_and_challenge_state() {
    let cargo_toml = include_str!("../Cargo.toml");
    let lib_rs = include_str!("../src/lib.rs");
    let wasm_api = include_str!("../src/wasm_api.rs");

    assert!(
        cargo_toml.contains("crate-type = [\"cdylib\", \"rlib\"]")
            && cargo_toml.contains("[target.'cfg(target_arch = \"wasm32\")'.dependencies]")
            && cargo_toml.contains("wasm-bindgen"),
        "crate should be configured for wasm-bindgen cdylib builds"
    );
    assert!(
        lib_rs.contains("#[cfg(target_arch = \"wasm32\")]")
            && lib_rs.contains("mod wasm_api;")
            && lib_rs.contains("pub fn scenario_summaries()")
            && lib_rs.contains("pub fn challenge_state_message"),
        "shared Rust game logic should be available to the wasm module"
    );
    assert!(
        wasm_api.contains("#[wasm_bindgen]")
            && wasm_api.contains("pub struct WasmGame")
            && wasm_api.contains("pub fn scenarios_json()")
            && wasm_api.contains("pub fn challenge_state_json")
            && wasm_api.contains("pub fn run_script_json")
            && wasm_api.contains("run_script(scenario_id, &script)")
            && wasm_api.contains("scripts_by_scenario"),
        "wasm API should expose local scenario loading, script execution, challenge state, and append-session state"
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
    assert!(
        scenarios
            .iter()
            .all(|scenario| scenario["upcoming"] == false),
        "all current scenarios should be unflagged as upcoming metadata"
    );
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
    assert!(
        arena_two.get("market").is_none(),
        "non-market puzzles should not expose empty market UI data"
    );
    assert!(
        arena_two.get("mail").is_none(),
        "non-mail puzzles should not expose empty mail UI data"
    );

    let market_one = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "05-auction-negative-price")
        .expect("market puzzle exists");
    assert_eq!(market_one["market"]["gold"], 100);
    assert!(market_one["market"]["listings"]
        .as_array()
        .is_some_and(|listings| listings.iter().any(|listing| {
            listing["id"] == 11
                && listing["item"] == "Phoenix Mount"
                && listing["price"] == 500
                && listing["stock"] == 1
                && listing["sprite"] == "mount"
        })));
    assert!(
        market_one.get("mail").is_none(),
        "pure market puzzles should not expose mail UI data"
    );

    let market_three = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "07-auction-cancel-refund-dupe")
        .expect("market/mail puzzle exists");
    assert_eq!(
        market_three["example_script"], "send ClaimMailbox { mail: 1 }\n",
        "market-3 should start from the obvious claim-proceeds action"
    );
    assert_eq!(market_three["title"], "market-3");
    let market_three_listing = &market_three["market"]["listings"][0];
    assert_eq!(market_three_listing["id"], 31);
    assert_eq!(market_three_listing["item"], "Copper Charm");
    assert_eq!(market_three_listing["sprite"], "gem");
    assert_eq!(
        market_three_listing["status"], "pending",
        "the visible UI-cancelable listing is a non-objective item"
    );
    assert_eq!(
        market_three_listing["cancel_packet"], "CancelListing { listing: 31 }",
        "the visible listing exposes a cancel action for UI packet gameplay"
    );
    let market_three_sold_listing = market_three["market"]["listings"]
        .as_array()
        .expect("market 3 listings are an array")
        .iter()
        .find(|listing| listing["id"] == 32)
        .expect("market 3 shows the completed sword sale as sold");
    assert_eq!(market_three_sold_listing["item"], "Listed Sword");
    assert_eq!(market_three_sold_listing["sprite"], "blade");
    assert_eq!(market_three_sold_listing["status"], "sold");
    assert_eq!(market_three_sold_listing["note"], "sale proceeds mailed");
    assert!(
        market_three_sold_listing.get("cancel_packet").is_none()
            || market_three_sold_listing["cancel_packet"].is_null(),
        "the hidden objective sword is not the visible UI cancel target"
    );
    // Before canceling, only the sale proceeds are waiting. The returned-item
    // mail is created by the cancel path, so it must not be pre-populated.
    let market_three_mail = market_three["mail"]["messages"]
        .as_array()
        .expect("market 3 exposes mail");
    assert_eq!(
        market_three_mail.len(),
        1,
        "only the sale mail exists until the listing is canceled"
    );
    assert!(market_three_mail.iter().any(|message| {
        message["id"] == 1 && message["attachment"] == "Gold" && message["status"] == "unclaimed"
    }));
    assert!(
        !market_three_mail.iter().any(|message| message["id"] == 2),
        "the returned-sword mail is a consequence of canceling, not initial state"
    );

    let post_office = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "08-dupe-mail-desync")
        .expect("post-office puzzle exists");
    assert!(
        post_office.get("market").is_none(),
        "mail-only puzzles should not expose market UI data"
    );
    assert_eq!(
        post_office["mail"]["messages"][0]["subject"],
        "Draft slot #1"
    );
    assert_eq!(post_office["mail"]["messages"][0]["status"], "draft");
}

#[tokio::test]
async fn api_scene_entities_expose_monster_traits_and_combat_stats() {
    let payload = api_scenarios_payload().await;
    let scenarios = payload.as_array().expect("/api/scenarios returns an array");

    let arena_two = scenarios
        .iter()
        .find(|scenario| scenario["id"] == "02-arena-fight-while-dead")
        .expect("arena 2 exists");
    let arena_monster = arena_two["scene"]["entities"]
        .as_array()
        .expect("arena 2 entities are an array")
        .iter()
        .find(|entity| entity["sprite"] == "monster")
        .expect("arena 2 has a monster entity");
    assert_eq!(arena_monster["type"], "monster");
    assert!(arena_monster["traits"]
        .as_array()
        .is_some_and(|traits| traits.iter().any(|value| value == "monster")));
    assert_eq!(arena_monster["hp"], 160);
    assert_eq!(arena_monster["max_hp"], 160);
}

#[test]
fn scene_renderer_uses_entity_traits_for_monster_overlays() {
    let scene_js = include_str!("../client/scene.js");
    assert!(scene_js.contains("function isMonsterEntity"));
    assert!(scene_js.contains("entity.traits"));
    assert!(scene_js.contains("drawEntityTraitBadges"));
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
        .is_some_and(|lesson| lesson.contains("client-declared material counts")));
}

#[test]
fn solved_recaps_do_not_offer_fix_tips() {
    for scenario in all_scenarios() {
        let lesson = scenario.lesson();
        assert!(
            !lesson.contains("Fix:")
                && !lesson.to_lowercase().contains("server-authoritative")
                && !lesson.to_lowercase().contains("defensive note"),
            "{} solve recap reads like remediation guidance: {}",
            scenario.id(),
            lesson
        );
    }
}

#[test]
fn scenario_reference_docs_do_not_offer_fix_tips() {
    let scenarios_dir = format!("{}/scenarios", env!("CARGO_MANIFEST_DIR"));
    for entry in fs::read_dir(scenarios_dir).expect("scenarios directory is readable") {
        let entry = entry.expect("scenario directory entry is readable");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("scenario markdown is readable");
        let lower = content.to_lowercase();
        assert!(
            !content.contains("## Defensive note")
                && !lower.contains("real-world fix")
                && !lower.contains("fix tips"),
            "{} should keep post-solve copy in CTF recap language",
            path.display()
        );
    }
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
    let initial: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert_eq!(initial["type"], "challenge_state");
    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "01-first-blood-batch",
                "script": "send_batch {\n  Attack { target: 1 }\n  Attack { target: 1 }\n  Attack { target: 1 }\n}\n"
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
async fn websocket_packet_actions_return_stateful_server_notifications() {
    let store = Store::memory().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app_with_store(store)).await.unwrap();
    });

    let (mut socket, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();
    let initial: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert_eq!(initial["type"], "challenge_state");
    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "07-auction-cancel-refund-dupe",
                "script": "send CancelListing { listing: 31 }\n",
                "append": false
            })
            .to_string(),
        ))
        .await
        .unwrap();
    let first_payload: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert!(first_payload["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| {
            event["kind"] == "server"
                && event["name"] == "MailCreated"
                && event["fields"]["mail"] == 2
                && event["fields"]["attachment"] == "Copper Charm"
        }));

    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "07-auction-cancel-refund-dupe",
                "script": "send ClaimMailbox { mail: 2 }\n",
                "append": true
            })
            .to_string(),
        ))
        .await
        .unwrap();
    let second_payload: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert!(second_payload["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| {
            event["kind"] == "server"
                && event["name"] == "MailClaimed"
                && event["fields"]["mail"] == 2
        }));
    assert!(second_payload["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| {
            event["kind"] == "server"
                && event["name"] == "InventoryAdded"
                && event["fields"]["item"] == "Copper Charm"
                && event["fields"]["sprite"] == "gem"
        }));

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn websocket_streams_challenge_state_on_connect_and_after_completion() {
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
        .insert("Cookie", "ph_uid=state-user".parse().unwrap());
    let (mut socket, _) = connect_async(request).await.unwrap();

    let initial: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert_eq!(initial["type"], "challenge_state");
    let initial_challenges = initial["challenges"].as_array().unwrap();
    assert!(initial_challenges.iter().any(|challenge| {
        challenge["id"] == "01-first-blood-batch"
            && challenge["completed"] == false
            && challenge["enabled"] == true
            && challenge["upcoming"] == false
            && challenge["status"] == "available"
    }));

    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "01-first-blood-batch",
                "script": "send_batch {\n  Attack { target: 1 }\n  Attack { target: 1 }\n  Attack { target: 1 }\n}\n"
            })
            .to_string(),
        ))
        .await
        .unwrap();

    let result: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert_eq!(result["type"], "run_result");
    assert_eq!(result["outcome"], "win");

    let updated: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert_eq!(updated["type"], "challenge_state");
    let updated_challenges = updated["challenges"].as_array().unwrap();
    assert!(updated_challenges.iter().any(|challenge| {
        challenge["id"] == "01-first-blood-batch"
            && challenge["completed"] == true
            && challenge["enabled"] == true
            && challenge["upcoming"] == false
            && challenge["status"] == "completed"
    }));
    assert!(updated_challenges.iter().any(|challenge| {
        challenge["id"] == "02-arena-fight-while-dead"
            && challenge["completed"] == false
            && challenge["enabled"] == true
            && challenge["upcoming"] == false
            && challenge["status"] == "available"
    }));

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
    let initial: Value =
        serde_json::from_str(&socket.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert_eq!(initial["type"], "challenge_state");
    socket
        .send(Message::Text(
            json!({
                "type": "run_script",
                "scenario_id": "01-first-blood-batch",
                "script": "send_batch {\n  Attack { target: 1 }\n  Attack { target: 1 }\n  Attack { target: 1 }\n}\n"
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
