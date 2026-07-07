use crate::protocol::{Outcome, PacketEvent, RunResult};
use crate::scenarios::find_scenario;
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct ClientEvent {
    pub t: u64,
    pub name: String,
    pub fields: Map<String, Value>,
}

#[derive(Debug, Clone)]
pub struct ScriptProgram {
    pub events: Vec<ClientEvent>,
    pub max_time: u64,
}

pub fn run_script(scenario_id: &str, script: &str) -> RunResult {
    let Some(scenario) = find_scenario(scenario_id) else {
        return RunResult::error(
            scenario_id.to_string(),
            format!("unknown scenario: {scenario_id}"),
        );
    };

    let program = match compile_script(script) {
        Ok(program) => program,
        Err(err) => {
            return RunResult::error(scenario_id.to_string(), format!("parse error: {err}"))
        }
    };

    let won = scenario.check_win(&program.events);
    let mut events: Vec<PacketEvent> = program
        .events
        .iter()
        .map(|event| PacketEvent {
            t: event.t,
            kind: "client".to_string(),
            name: event.name.clone(),
            fields: event.fields.clone(),
        })
        .collect();

    // Authoritative server-side notifications the world emits in response to the
    // player's packets (listing removed, mail created, ...). The client renders
    // these to update the market/mail UI instead of mutating state locally.
    events.extend(scenario.notifications(&program.events));

    let visible_scenario = scenario.player_title();

    if won {
        events.push(PacketEvent {
            t: program.max_time,
            kind: "server".to_string(),
            name: "ObjectiveComplete".to_string(),
            fields: Map::from_iter([("scenario".to_string(), json!(visible_scenario))]),
        });
    } else {
        events.push(PacketEvent {
            t: program.max_time,
            kind: "server".to_string(),
            name: "ObjectiveFailed".to_string(),
            fields: Map::from_iter([("scenario".to_string(), json!(visible_scenario))]),
        });
    }

    let mut state = Map::from_iter([
        ("scenario".to_string(), json!(visible_scenario)),
        ("title".to_string(), json!(visible_scenario)),
        ("objective".to_string(), json!(scenario.objective())),
        ("packets_sent".to_string(), json!(program.events.len())),
        (
            "result".to_string(),
            json!(if won { "win" } else { "lose" }),
        ),
    ]);
    if won {
        state.insert("lesson".to_string(), json!(scenario.lesson()));
    }

    RunResult {
        message_type: "run_result".to_string(),
        ok: true,
        scenario_id: scenario_id.to_string(),
        outcome: if won { Outcome::Win } else { Outcome::Lose },
        time_ms: program.max_time,
        state: Value::Object(state),
        events,
        error: None,
    }
}

pub fn compile_script(script: &str) -> Result<ScriptProgram, String> {
    let mut lines = Vec::new();
    for raw in script.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if !line.is_empty() {
            lines.push(line.to_string());
        }
    }

    let mut parser = Parser { lines, idx: 0 };
    let mut ctx = ExecCtx::default();
    parser.exec_block(&mut ctx, None, None, false)?;
    if parser.idx < parser.lines.len() {
        return Err(format!(
            "unexpected trailing input: {}",
            parser.lines[parser.idx]
        ));
    }
    let max_time = ctx.events.iter().map(|e| e.t).max().unwrap_or(ctx.now);
    Ok(ScriptProgram {
        events: ctx.events,
        max_time,
    })
}

#[derive(Default)]
struct ExecCtx {
    now: u64,
    events: Vec<ClientEvent>,
}

struct Parser {
    lines: Vec<String>,
    idx: usize,
}

impl Parser {
    fn exec_block(
        &mut self,
        ctx: &mut ExecCtx,
        forced_time: Option<u64>,
        loop_var: Option<(&str, i64)>,
        allow_packet_literals: bool,
    ) -> Result<(), String> {
        while self.idx < self.lines.len() {
            let line = self.lines[self.idx].clone();
            if line == "}" {
                self.idx += 1;
                return Ok(());
            }

            if line == "batch {" {
                return Err(
                    "batch was renamed to send_batch; packet lines inside send_batch omit send"
                        .to_string(),
                );
            }

            if line == "send_batch {" {
                self.idx += 1;
                let t = forced_time.unwrap_or(ctx.now);
                self.exec_block(ctx, Some(t), loop_var, true)?;
                continue;
            }

            if let Some(t) = parse_at_header(&line)? {
                self.idx += 1;
                self.exec_block(ctx, Some(t), loop_var, allow_packet_literals)?;
                ctx.now = ctx.now.max(t);
                continue;
            }

            if let Some((var, start, end)) = parse_for_header(&line)? {
                self.idx += 1;
                let body_start = self.idx;
                let body_end = find_matching_end(&self.lines, body_start)?;
                for value in start..end {
                    let mut nested = Parser {
                        lines: self.lines[body_start..body_end].to_vec(),
                        idx: 0,
                    };
                    nested.exec_block(
                        ctx,
                        forced_time,
                        Some((&var, value)),
                        allow_packet_literals,
                    )?;
                }
                self.idx = body_end + 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("sleep ") {
                if forced_time.is_some() {
                    return Err("sleep is not allowed inside send_batch/at blocks".to_string());
                }
                let value = substitute(rest.trim(), loop_var)
                    .parse::<u64>()
                    .map_err(|_| format!("invalid sleep duration: {}", rest.trim()))?;
                ctx.now += value;
                self.idx += 1;
                continue;
            }

            if allow_packet_literals && line.contains('{') {
                if line.starts_with("send ") {
                    return Err(
                        "packet lines inside send_batch omit send; use Packet { ... }".to_string(),
                    );
                }
                let event = parse_send(&line, forced_time.unwrap_or(ctx.now), loop_var)?;
                ctx.events.push(event);
                self.idx += 1;
                continue;
            }

            if let Some(rest) = line.strip_prefix("send ") {
                let event = parse_send(rest, forced_time.unwrap_or(ctx.now), loop_var)?;
                ctx.events.push(event);
                self.idx += 1;
                continue;
            }

            if line.starts_with("let ") && line.contains(" await ") {
                // Prototype await: scenario solutions can read packet feeds later; for now this is a no-op
                // that lets authored scripts keep their intended shape.
                self.idx += 1;
                continue;
            }

            if line.starts_with("await ") {
                self.idx += 1;
                continue;
            }

            return Err(format!("unsupported statement: {line}"));
        }
        Ok(())
    }
}

fn parse_for_header(line: &str) -> Result<Option<(String, i64, i64)>, String> {
    if !line.starts_with("for ") {
        return Ok(None);
    }
    let Some(prefix) = line.strip_suffix("{") else {
        return Err(format!("for loop must end with '{{': {line}"));
    };
    let rest = prefix.trim_start_matches("for ").trim();
    let Some((var, range)) = rest.split_once(" in ") else {
        return Err(format!("invalid for loop: {line}"));
    };
    let Some((start, end)) = range.trim().split_once("..") else {
        return Err(format!("invalid for range: {line}"));
    };
    Ok(Some((
        var.trim().to_string(),
        start
            .trim()
            .parse()
            .map_err(|_| format!("invalid range start: {line}"))?,
        end.trim()
            .parse()
            .map_err(|_| format!("invalid range end: {line}"))?,
    )))
}

fn parse_at_header(line: &str) -> Result<Option<u64>, String> {
    if !line.starts_with("at(") {
        return Ok(None);
    }
    let Some(prefix) = line.strip_suffix("{") else {
        return Err(format!("at block must end with '{{': {line}"));
    };
    let inner = prefix
        .trim()
        .strip_prefix("at(")
        .and_then(|s| s.strip_suffix(')'));
    let Some(inner) = inner else {
        return Err(format!("invalid at block: {line}"));
    };
    Ok(Some(
        inner
            .trim()
            .parse()
            .map_err(|_| format!("invalid at time: {line}"))?,
    ))
}

fn find_matching_end(lines: &[String], start: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    for (offset, line) in lines[start..].iter().enumerate() {
        if line.ends_with('{') {
            depth += 1;
        }
        if line == "}" {
            if depth == 0 {
                return Ok(start + offset);
            }
            depth -= 1;
        }
    }
    Err("unterminated block".to_string())
}

fn parse_send(rest: &str, t: u64, loop_var: Option<(&str, i64)>) -> Result<ClientEvent, String> {
    let Some((name, fields_raw)) = rest.split_once('{') else {
        return Err(format!("send missing packet fields: send {rest}"));
    };
    let fields_raw = fields_raw.trim();
    let Some(fields_raw) = fields_raw.strip_suffix('}') else {
        return Err(format!("send missing closing brace: send {rest}"));
    };
    let fields = parse_fields(fields_raw, loop_var)?;
    Ok(ClientEvent {
        t,
        name: name.trim().to_string(),
        fields,
    })
}

fn parse_fields(raw: &str, loop_var: Option<(&str, i64)>) -> Result<Map<String, Value>, String> {
    let mut fields = Map::new();
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(fields);
    }

    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((key, value)) = part.split_once(':') else {
            return Err(format!("field missing ':' in {part}"));
        };
        let key = key.trim();
        let value = substitute(value.trim(), loop_var);
        if key.is_empty() || value.trim().is_empty() {
            return Err(format!("invalid field: {part}"));
        }
        fields.insert(key.to_string(), parse_value(value.trim())?);
    }
    Ok(fields)
}

fn substitute(input: &str, loop_var: Option<(&str, i64)>) -> String {
    if let Some((name, value)) = loop_var {
        if input == name {
            return value.to_string();
        }
    }
    input.to_string()
}

fn parse_value(value: &str) -> Result<Value, String> {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return Ok(json!(value.trim_matches('"')));
    }
    if value == "true" {
        return Ok(json!(true));
    }
    if value == "false" {
        return Ok(json!(false));
    }
    if let Ok(int_value) = value.parse::<i64>() {
        return Ok(json!(int_value));
    }
    if value.contains('{') || value.contains('[') || value.contains(']') {
        return Err(format!(
            "complex literal not supported in prototype parser: {value}"
        ));
    }
    Ok(json!(value))
}

pub fn field_i64(event: &ClientEvent, key: &str) -> Option<i64> {
    event.fields.get(key)?.as_i64()
}

pub fn field_str<'a>(event: &'a ClientEvent, key: &str) -> Option<&'a str> {
    event.fields.get(key)?.as_str()
}
