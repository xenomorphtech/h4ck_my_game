# Packet Hacker — Game Design & Technical Spec

A puzzle/hacking training game. Each scenario is a network protocol puzzle:
you cannot win by "playing fair" — you must craft, batch, and time raw packets
using a small scripting language to exploit the server's rules.

---

## 1. Concept

You play a client talking to a game server over a (simulated) packet protocol.
The server enforces rules that make each objective **impossible via normal play**.
Victory requires understanding the protocol and writing a script that abuses
timing, batching, ordering, or malformed fields.

Think: "Shenzhen I/O meets a packet sniffer meets a MMO exploit sandbox."

---

## 2. Screen Layout

```
+----------------------+--------------------------------------------------+
|                      |                                                  |
|   PACKET LIST        |            SCENARIO / WORLD VIEW                  |
|   (left panel)       |            (center stage)                        |
|                      |                                                  |
|  known packet types, |  visual state of the scenario:                   |
|  their fields,       |  the monster, HP bars, entities, timers,         |
|  direction (C->S,    |  and a live packet log / event ticker.           |
|  S->C), and a        |                                                  |
|  "captured" feed of  |  Objective banner shown at top:                   |
|  recent traffic.     |  "Kill the monster."                             |
|                      |                                                  |
+----------------------+--------------------------------------------------+
|  SCRIPT EDITOR (bottom panel)                              [ Run ][Stop]|
|                                                                         |
|  for i in 1..5 {                                                        |
|      send Attack { target: 1, power: 10 }                               |
|      sleep 20                                                           |
|  }                                                                      |
|                                                                         |
|  > console output / runtime errors / assertion results                  |
+-------------------------------------------------------------------------+
```

### 2.1 Left — Packet List
- Lists every packet type available in the current scenario.
- Each entry expands to show fields, types, defaults, and direction.
  - `C->S` (client to server, you can `send` these)
  - `S->C` (server to client, you can `on`-handle / observe these)
- Doubles as a live capture feed: incoming/outgoing packets scroll here with
  timestamps (relative to script start, in ms), so you can reverse-engineer
  the protocol by watching real traffic.
- Clicking a packet inserts a `send` template into the editor at the cursor.

### 2.2 Center — Scenario View
- Renders the scenario's entities and their state (HP, position, cooldowns).
- Top: objective banner + win/lose state.
- Bottom strip: event ticker (damage numbers, deaths, server messages).
- A timeline scrubber (ms resolution) for post-run inspection: replay what
  packets went out and came in, frame by frame. Essential for a timing game.

### 2.3 Bottom — Script Editor
- Text editor with syntax highlighting for the scripting language.
- `Run` compiles + executes the script against the live scenario.
- `Stop` aborts. Console shows output, errors, and objective assertions.
- Re-running resets the scenario to its initial state (deterministic).

---

## 3. Scripting Language — "Netscript"

Design goals: **ML-flavored, comfortable to write, tiny, no performance focus.**
Interpreted, dynamically typed, single-threaded with a cooperative virtual clock.
Whitespace-insensitive; braces for blocks; expression-oriented.

### 3.1 Feel
- Familiar C-brace blocks for loops/branches (comfortable, low surprise).
- ML-style `let` bindings, expression-orientation, lightweight lambdas,
  pattern matching, and immutable-by-default values.
- No semicolons required; newlines separate statements.

### 3.2 Core Syntax

Bindings (immutable by default, `let mut` to allow reassignment):
```
let x = 10
let name = "goblin"
let mut hp = 100
hp = hp - 5
```

Packet literals (record syntax, ML-ish; missing fields take declared defaults):
```
Attack { target: 1, power: 10 }
Move { x: 4, y: 7 }
```

Send / receive:
```
send Attack { target: 1, power: 10 }        # enqueue+flush one packet
sleep 20                                     # advance virtual clock 20 ms
```

Loops:
```
for i in 1..5 {          # inclusive-exclusive range: 1,2,3,4
    send Attack { target: 1 }
    sleep 20
}

while alive(monster) {
    send Probe {}
    sleep 100
}
```

Conditionals (also usable as expressions):
```
if hp < 50 {
    send Heal {}
} else {
    send Attack { target: 1 }
}

let label = if hp < 50 { "danger" } else { "ok" }
```

Functions / lambdas (ML-style, last expression is the return value):
```
let volley = fn (n) {
    for i in 1..n { send Attack { target: 1 } }
}
volley(3)

let double = fn (x) { x * 2 }
```

Pattern matching on incoming packets and values:
```
match packet {
    Damage { amount } if amount > 50 => log("big hit: {amount}")
    Damage { amount }                => log("hit: {amount}")
    Death { id }                     => log("entity {id} died")
    _                                => ()
}
```

### 3.3 The Killer Feature: Batching & Timing

This is what makes the game work. The runtime distinguishes between
**enqueue** and **flush**, so you can control exactly how packets hit the wire
within a single simulation tick.

```
send_batch {
    Attack { target: 1 }
    Attack { target: 1 }
    Attack { target: 1 }
}                # all three flushed in the SAME tick (0 ms apart)
```

- Bare `send` outside a `send_batch` flushes immediately (its own tick).
- Inside `send_batch { ... }`, packet literal lines omit `send` and flush
  atomically together when the block closes — they arrive server-side in the
  same packet frame.
- `at(t) { ... }` schedules a batch to flush at absolute virtual time `t` ms.
- `parallel { a; b }` interleaves two logical timelines (advanced scenarios).

```
at(0)   { send Attack { target: 1 } }
at(0)   { send Attack { target: 1 } }   # coincident with the above
at(250) { send Attack { target: 1 } }
```

Timing/async primitives:
```
sleep 20              # advance virtual clock
now()                 # current virtual time in ms
await Death { id: 1 } # block until a matching S->C packet arrives (or timeout)
on Damage { amount } => log(amount)   # register a background handler
```

### 3.4 Built-ins (scenario-agnostic helpers)
```
log(msg)              # print to console (supports "{var}" interpolation)
assert(cond, msg)     # fail the run with a message if cond is false
now()                 # virtual time, ms
random(a, b)          # deterministic PRNG seeded per-run
alive(entity)         # query helper exposed by scenario
hp(entity)            # query helper exposed by scenario
```

### 3.5 Types
- `Int`, `Float`, `Bool`, `String`, `Unit` (`()`)
- `Range` (`1..5`)
- `Packet` (record value tied to a declared packet schema)
- `Fn` (first-class functions/closures)
- Interpolated strings: `"hp is {hp} at {now()}ms"`

### 3.6 Execution Model
- Single script "main" timeline + optional background `on` handlers.
- **Virtual clock**: nothing is real-time. `sleep`, `at`, `await` move a
  deterministic simulated clock. The whole run is reproducible (seeded PRNG,
  fixed server logic), which is what makes timing puzzles fair and solvable.
- Server logic runs as a deterministic step function reacting to the ordered,
  timestamped stream of client packets the script produced.
- Hard limits: max simulated duration, max packets, max instructions
  (prevents infinite loops from hanging the UI). Exceeding = run error.

---

## 4. Scenario Model

A scenario is data-driven. It declares packet schemas, initial world state,
server rules, and a win condition.

### 4.1 Scenario Definition (author-facing, e.g. JSON/DSL)
```
scenario "First Blood" {
    objective: "Kill the monster."

    packets {
        C->S Attack { target: Int, power: Int = 10 }
        C->S Move   { x: Int, y: Int }
        S->C Damage { source: Int, target: Int, amount: Int }
        S->C Death  { id: Int }
    }

    entities {
        monster { id: 1, hp: 100 }
        player  { id: 0, hp: 100 }
    }

    rules {
        # The trap: any single attack triggers a fatal 250ms retaliation.
        on Attack (a) {
            deal_damage(to: a.target, amount: 40)
            schedule(250) {           # retaliation window
                deal_damage(to: 0, amount: 999)   # player dies -> lose
            }
        }
        # The exploit: 3 attacks in the SAME frame stack before retaliation
        # can register, and 3*40 = 120 >= 100 -> monster dies first.
        # Because retaliation is scheduled per-attack at +250ms, batching all
        # three at t=0 kills the monster at t=0, before any retaliation fires.
    }

    win:  monster.hp <= 0
    lose: player.hp  <= 0
}
```

### 4.2 The Intended Solution (author's reference)
```
send_batch {
    Attack { target: 1 }
    Attack { target: 1 }
    Attack { target: 1 }
}
# monster takes 120 at t=0, dies. retaliation scheduled for t=250 never
# matters because the scenario already resolved to WIN.
```
The naive attempt (`send Attack` once, or with `sleep 20` between) loses,
because the first attack's +250ms retaliation lands before you can stack 100 dmg.

---

## 5. Example Scenario Ladder (teaches one trick each)

1. **First Blood** — batching. (the retaliation puzzle above)
2. **Race Condition** — send a `Buy` and `Use` in the same frame so the item
   exists for `Use` before the server debits gold (TOCTOU).
3. **Rapid Fire** — server rate-limits to 1 packet / 50ms *by timestamp*, but
   doesn't dedupe identical timestamps: flood 10 attacks `at(0)`.
4. **Rollback** — `Move` is validated against last-acked position; send moves
   faster than the server acks to teleport past a wall.
5. **Replay** — capture a signed `Loot` packet from the feed and re-send it.
6. **Desync** — two entities, alternate `parallel` timelines to keep both
   monster retaliations from ever aligning.
7. **Integer Overflow** — `power: 2147483647 + 1` wraps negative; heal a boss
   into death, or set your own hp absurdly high.

Each scenario's left panel shows only the packets relevant to it; discovering
the exploit = reading schemas + watching the capture feed + experimenting.

A fuller draft scenario catalog lives in `scenarios/`, with one markdown file per
puzzle covering auction-house bugs, duplication bugs, target validation bugs,
rate-limit bugs, replay bugs, movement/telehacking bugs, crafting bugs, and
progression bugs.

---

## 6. Win/Lose & Feedback
- **Win**: objective predicate true -> success screen, show packet timeline,
  offer "optimal" stats (fewest packets, lowest sim-time) as replay-value.
- **Lose**: predicate for lose true OR run ends without win -> show why
  (e.g. "player died at t=250ms from retaliation") + let them edit & re-run.
- **Assertions**: authors can attach hints that fire in console on common
  wrong approaches ("you attacked once — what happens 250ms later?").

---

## 7. Technical Architecture

### 7.1 Components
- **Editor** — code editor (syntax highlight, error squiggles). Web:
  CodeMirror/Monaco. Desktop: whatever the host toolkit provides.
- **Netscript engine** — lexer -> parser (AST) -> tree-walking interpreter.
  No JIT, no bytecode needed; performance is a non-goal.
- **Simulation core** — deterministic event loop:
  1. interpreter emits ordered `(t_ms, packet)` client events,
  2. server rule engine steps through events + its own scheduled callbacks,
  3. world state mutates, S->C packets generated,
  4. `on`/`await` handlers in the script observe S->C stream.
- **Renderer** — draws center scenario view + left feed + timeline scrubber
  from world-state snapshots per tick.
- **Scenario loader** — parses scenario definitions (packets/entities/rules/
  win-lose) into the sim core.

### 7.2 Determinism Contract
- Single seeded PRNG. No wall-clock. No real I/O during a run.
- Same script + same scenario => identical outcome, always.
- This is required for: fair puzzles, replay/timeline, and shareable solutions.

### 7.3 Suggested Stack (proposal, open to change)
- Web app: TypeScript + a canvas/WebGL renderer for the center stage,
  React (or similar) for panels, Monaco for the editor.
- Engine written in plain TS so it can also run headless for tests/CI.
- Scenarios authored as declarative files (JSON or a small DSL) so new levels
  need no engine changes.

---

## 8. Netscript Grammar (EBNF sketch)

```
program     = { statement } ;
statement   = let | assign | send | send_batch | sleep | for | while | if
            | at | expr_stmt ;

let         = "let" [ "mut" ] ident "=" expr ;
assign      = ident "=" expr ;
send        = "send" packet ;
send_batch  = "send_batch" "{" { packet | for } "}" ;
sleep       = "sleep" expr ;
for         = "for" ident "in" expr "{" { statement } "}" ;
while       = "while" expr "{" { statement } "}" ;
if          = "if" expr block [ "else" (if | block) ] ;
at          = "at" "(" expr ")" block ;
block       = "{" { statement } "}" ;

expr        = literal | ident | call | binop | packet | lambda
            | match | range | if | "(" expr ")" ;
lambda      = "fn" "(" [ params ] ")" block ;
match       = "match" expr "{" { arm } "}" ;
arm         = pattern [ "if" expr ] "=>" (expr | block) ;
packet      = ident "{" [ field { "," field } ] "}" ;
field       = ident ":" expr ;
range       = expr ".." expr ;
call        = ident "(" [ args ] ")" ;
literal     = int | float | string | bool | "(" ")" ;
```

---

## 9. Open Questions
- Match-block scheduling: how do background `on` handlers order vs. main
  timeline when both act at the same `t`? (Proposal: main first, then handlers,
  stable by registration order.)
- Should `send_batch` allow nested `sleep`? (Proposal: no — `sleep` inside
  `send_batch` is a compile error; batches are instantaneous.)
- Scenario authoring: JSON vs. dedicated DSL? DSL reads better (section 4) but
  costs a second parser.
- Multiplayer/adversarial scenarios later? (Two scripts vs. each other.)
- How much protocol should be hidden vs. shown in the left panel by default?
  (Proposal: show schemas, hide exploit-relevant server rules; force discovery.)
