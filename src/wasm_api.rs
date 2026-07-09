use crate::protocol::{Outcome, RunResult, RunScriptRequest};
use crate::{challenge_state_message, run_script, scenario_summaries};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmGame {
    scripts_by_scenario: HashMap<String, String>,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            scripts_by_scenario: HashMap::new(),
        }
    }

    pub fn scenarios_json(&self) -> String {
        scenarios_json()
    }

    pub fn challenge_state_json(&self, completed_json: &str) -> String {
        challenge_state_json(completed_json)
    }

    pub fn run_script_json(&mut self, scenario_id: &str, script: &str, append: bool) -> String {
        let script = if append {
            let saved = self
                .scripts_by_scenario
                .entry(scenario_id.to_string())
                .or_default();
            saved.push_str(script);
            saved.clone()
        } else {
            self.scripts_by_scenario
                .insert(scenario_id.to_string(), script.to_string());
            script.to_string()
        };

        to_json(&run_script(scenario_id, &script))
    }

    pub fn handle_run_request_json(&mut self, request_json: &str) -> String {
        let result = match serde_json::from_str::<RunScriptRequest>(request_json) {
            Ok(request) if request.message_type == "run_script" => {
                self.run_script_json(&request.scenario_id, &request.script, request.append)
            }
            Ok(request) => to_json(&RunResult::error(
                request.scenario_id,
                format!("unsupported message type: {}", request.message_type),
            )),
            Err(err) => to_json(&RunResult::error("", format!("invalid json: {err}"))),
        };
        result
    }

    pub fn reset_script_session(&mut self, scenario_id: &str) {
        self.scripts_by_scenario.remove(scenario_id);
    }
}

#[wasm_bindgen]
pub fn scenarios_json() -> String {
    to_json(&scenario_summaries())
}

#[wasm_bindgen]
pub fn challenge_state_json(completed_json: &str) -> String {
    to_json(&challenge_state_message(&completed_ids(completed_json)))
}

fn completed_ids(completed_json: &str) -> Vec<String> {
    let value = serde_json::from_str::<Value>(completed_json).unwrap_or(Value::Null);
    let mut ids = match value {
        Value::Array(items) => strings_from_values(items),
        Value::Object(mut object) => object
            .remove("completed")
            .and_then(|value| value.as_array().cloned())
            .map(strings_from_values)
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    ids.sort();
    ids.dedup();
    ids
}

fn strings_from_values(items: Vec<Value>) -> Vec<String> {
    items
        .into_iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect()
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|err| {
        let result = RunResult {
            message_type: "run_result".to_string(),
            ok: false,
            scenario_id: String::new(),
            outcome: Outcome::Error,
            time_ms: 0,
            state: Value::Object(Default::default()),
            events: Vec::new(),
            error: Some(format!("serialization error: {err}")),
        };
        serde_json::to_string(&result).expect("error result serializes")
    })
}
