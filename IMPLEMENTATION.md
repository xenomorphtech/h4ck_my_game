# Packet Hacker Implementation Design

This file defines the concrete build target for the first playable prototype.
It intentionally narrows the large game spec into a testable vertical slice:
Rust server, JSON-over-WebSocket protocol, deterministic scenario engine, tests
for every puzzle, and a browser client served by the Rust app.

## Goals

- Rust implementation.
- JSON WebSocket API.
- Deterministic scenario runner for all documented puzzles.
- Tests proving each puzzle's intended exploit wins and representative naive
  attempts lose.
- Browser client with:
  - scenario list on the left,
  - scenario/world state in the center,
  - script editor at the bottom,
  - run button and packet/event output.

## Non-goals for this prototype

- No real networking exploitation. This is a simulated training game.
- No high-performance VM/JIT.
- No multiplayer.
- No full Monaco/CodeMirror integration required yet; a textarea is enough.
- No complete Netscript language. Implement a small parser/interpreter sufficient
  for the scenario solutions and common experimentation.

## Project layout

```text
Cargo.toml
src/
  main.rs             # axum HTTP + websocket server entrypoint
  lib.rs              # exports for tests
  protocol.rs         # JSON request/response types
  engine.rs           # script parsing/execution + scenario runner
  scenarios.rs        # scenario definitions and bug simulations
  static_files.rs     # serves embedded or filesystem client assets
client/
  index.html
  style.css
  app.js
tests/
  scenario_runs.rs    # all puzzle exploit/naive tests through public API
```

## HTTP/WebSocket API

### HTTP

- `GET /` -> `client/index.html`
- `GET /client/style.css`
- `GET /client/app.js`
- `GET /api/scenarios` -> scenario metadata JSON

### WebSocket

- Path: `GET /ws`
- Each frame is a JSON object.

Client -> server:

```json
{
  "type": "run_script",
  "scenario_id": "01-first-blood-batch",
  "script": "batch {\n  send Attack { target: 1 }\n}\n"
}
```

Server -> client:

```json
{
  "type": "run_result",
  "ok": true,
  "scenario_id": "01-first-blood-batch",
  "outcome": "win",
  "time_ms": 0,
  "state": { "player_hp": 100, "monster_hp": 0 },
  "events": [
    { "t": 0, "kind": "client", "name": "Attack", "fields": { "target": 1 } },
    { "t": 0, "kind": "server", "name": "Death", "fields": { "id": 1 } }
  ],
  "error": null
}
```

Error shape:

```json
{
  "type": "run_result",
  "ok": false,
  "scenario_id": "...",
  "outcome": "error",
  "time_ms": 0,
  "state": {},
  "events": [],
  "error": "parse error at line 3: ..."
}
```

## Script subset

Implement only what is needed by scenario solution scripts:

- `send Packet { field: value, ... }`
- `sleep <int>`
- `batch { ... }`
- `at(<int>) { ... }`
- `for i in A..B { ... }` with inclusive-exclusive range.
- `let name = await Packet { optional_field: value }`
- Field references from awaited packets in later sends: `g.signature`, `d.drop`.
- Packet field values:
  - integers, negative integers,
  - booleans,
  - strings,
  - arrays of packet-like records for crafting materials,
  - identifiers for scenario constants (`Pebble`, `DragonScale`, etc.).

Execution model:

- Scripts compile to timestamped client packet events.
- Bare `send` emits at current time.
- `sleep N` advances current time.
- `batch` executes child sends at the same parent timestamp; sleep inside batch
  should be rejected.
- `at(T)` executes child sends at absolute time `T`.
- `await` can be simplified: during script event generation, if the requested
  packet is produced by the scenario after prior events, bind the first matching
  server packet fields. This does not need real concurrency yet.

## Scenario runner design

Use one enum or registry entry per puzzle:

```rust
pub struct Scenario {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub difficulty: &'static str,
    pub objective: &'static str,
    pub packets: Vec<PacketSchema>,
    pub solution_script: &'static str,
    pub naive_script: &'static str,
    pub run: fn(&[ClientEvent]) -> RunResult,
}
```

The runner for each scenario may be hand-coded. This is acceptable for the
prototype and keeps the behavior easy to test against the puzzle docs. Each
runner consumes timestamped packets and returns `win`, `lose`, or `running/error`
with final state and event log.

## Required tests

For every scenario in `scenarios/README.md`:

- `solution_script` run through the same public `run_script(scenario_id, script)`
  API returns `outcome == Win`.
- `naive_script` returns anything other than `Win`.

Additional protocol tests:

- `GET /api/scenarios` lists all scenario ids.
- WebSocket accepts a valid `run_script` and returns a `run_result`.
- Unknown scenario returns a structured error.
- Parse errors return a structured error.

## Client behavior

- On load, fetch `/api/scenarios` and render scenario list on the left.
- Selecting a scenario:
  - shows title/objective/packet schemas in the center,
  - fills editor with its `solution_script` as a starter reference for now
    (later we can switch to a blank editor/hints).
- Run button opens/reuses `/ws`, sends `run_script`, and renders result.
- Event log shows timestamp, packet kind, packet name, and JSON fields.

## Acceptance checklist

- `cargo fmt --check` passes.
- `cargo test` passes.
- All 22 scenario solution tests pass.
- All 22 scenario naive tests fail to win.
- Server can be launched with `cargo run`.
- Browser assets are served from `/`.
- WebSocket smoke test succeeds against a live server.
