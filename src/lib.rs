pub mod engine;
pub mod protocol;
pub mod scenarios;
#[cfg(not(target_arch = "wasm32"))]
pub mod static_files;
#[cfg(not(target_arch = "wasm32"))]
pub mod store;
#[cfg(target_arch = "wasm32")]
mod wasm_api;

#[cfg(not(target_arch = "wasm32"))]
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
#[cfg(not(target_arch = "wasm32"))]
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use protocol::{ChallengeState, ChallengeStateMessage};
#[cfg(not(target_arch = "wasm32"))]
use protocol::{ProgressResponse, RunScriptRequest};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use tower_http::services::ServeDir;

pub use engine::run_script;
pub use protocol::{Outcome, RunResult};
pub use scenarios::{all_scenarios, Scenario};
#[cfg(not(target_arch = "wasm32"))]
pub use store::Store;

#[cfg(not(target_arch = "wasm32"))]
static NEXT_USER_ID: AtomicU64 = AtomicU64::new(1);

pub fn scenario_summaries() -> Vec<protocol::ScenarioSummary> {
    all_scenarios()
        .iter()
        .map(|scenario| protocol::ScenarioSummary::from(*scenario))
        .collect()
}

pub fn challenge_state_message(completed: &[String]) -> ChallengeStateMessage {
    let completed = completed.iter().map(String::as_str).collect::<HashSet<_>>();
    ChallengeStateMessage {
        message_type: "challenge_state".to_string(),
        challenges: all_scenarios()
            .iter()
            .map(|scenario| {
                let done = completed.contains(scenario.id());
                let upcoming = scenario.upcoming();
                ChallengeState {
                    id: scenario.id().to_string(),
                    enabled: done || !upcoming,
                    completed: done,
                    upcoming,
                    status: if done {
                        "completed"
                    } else if upcoming {
                        "upcoming"
                    } else {
                        "available"
                    }
                    .to_string(),
                }
            })
            .collect(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn app() -> Router {
    let path =
        std::env::var("PACKET_HACKER_DB").unwrap_or_else(|_| "packet_hacker.sqlite3".to_string());
    let store = Store::open(path).expect("open packet hacker sqlite database");
    app_with_store(store)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn app_with_store(store: Store) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/client/style.css", get(static_files::style))
        .route("/client/app.js", get(static_files::app_js))
        .route("/client/scene.js", get(static_files::scene_js))
        .route("/client/combat.js", get(static_files::combat_js))
        .route("/client/icons/:name", get(static_files::icon))
        .nest_service("/client/pkg", ServeDir::new("client/pkg"))
        .route("/api/scenarios", get(api_scenarios))
        .route("/api/progress", get(api_progress))
        .route("/ws", get(ws_handler))
        .with_state(store)
}

#[cfg(not(target_arch = "wasm32"))]
async fn index(State(store): State<Store>, headers: HeaderMap) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers).unwrap_or_else(new_user_id);
    let _ = store.ensure_user(&user_id);
    ([cookie_header(&user_id)], static_files::index().await).into_response()
}

#[cfg(not(target_arch = "wasm32"))]
async fn api_scenarios() -> Json<Vec<protocol::ScenarioSummary>> {
    Json(scenario_summaries())
}

#[cfg(not(target_arch = "wasm32"))]
async fn api_progress(State(store): State<Store>, headers: HeaderMap) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers).unwrap_or_else(new_user_id);
    match store
        .ensure_user(&user_id)
        .and_then(|_| store.completed_ids(&user_id))
    {
        Ok(completed) => (
            StatusCode::OK,
            [cookie_header(&user_id)],
            Json(ProgressResponse { user_id, completed }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn ws_handler(
    State(store): State<Store>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers).unwrap_or_else(new_user_id);
    let _ = store.ensure_user(&user_id);
    ws.on_upgrade(move |socket| handle_socket(socket, store, user_id))
}

#[cfg(not(target_arch = "wasm32"))]
async fn handle_socket(socket: WebSocket, store: Store, user_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut scripts_by_scenario = HashMap::<String, String>::new();
    if !send_challenge_state(&mut sender, &store, &user_id).await {
        return;
    }
    while let Some(Ok(message)) = receiver.next().await {
        let Message::Text(text) = message else {
            continue;
        };

        let result = match serde_json::from_str::<RunScriptRequest>(&text) {
            Ok(request) if request.message_type == "run_script" => {
                let script = if request.append {
                    let script = scripts_by_scenario
                        .entry(request.scenario_id.clone())
                        .or_default();
                    script.push_str(&request.script);
                    script.clone()
                } else {
                    scripts_by_scenario.insert(request.scenario_id.clone(), request.script.clone());
                    request.script.clone()
                };
                run_script(&request.scenario_id, &script)
            }
            Ok(request) => RunResult::error(
                request.scenario_id,
                format!("unsupported message type: {}", request.message_type),
            ),
            Err(err) => RunResult::error("".to_string(), format!("invalid json: {err}")),
        };

        let completed = result.outcome == protocol::Outcome::Win;
        if completed {
            let _ = store.mark_completed(&user_id, &result.scenario_id);
        }

        if let Ok(text) = serde_json::to_string(&result) {
            if sender.send(Message::Text(text)).await.is_err() {
                break;
            }
        }

        if completed && !send_challenge_state(&mut sender, &store, &user_id).await {
            break;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_challenge_state(
    sender: &mut SplitSink<WebSocket, Message>,
    store: &Store,
    user_id: &str,
) -> bool {
    let Ok(completed) = store.completed_ids(user_id) else {
        return true;
    };
    let message = challenge_state_message(&completed);
    let Ok(text) = serde_json::to_string(&message) else {
        return true;
    };
    sender.send(Message::Text(text)).await.is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn user_id_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        if name == "ph_uid" && valid_user_id(value) {
            Some(value.to_string())
        } else {
            None
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn valid_user_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[cfg(not(target_arch = "wasm32"))]
fn new_user_id() -> String {
    let counter = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("u{nanos:x}{counter:x}")
}

#[cfg(not(target_arch = "wasm32"))]
fn cookie_header(user_id: &str) -> (header::HeaderName, HeaderValue) {
    (
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "ph_uid={user_id}; Path=/; SameSite=Lax; Max-Age=31536000"
        ))
        .expect("safe cookie value"),
    )
}
