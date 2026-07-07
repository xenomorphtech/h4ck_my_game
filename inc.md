# Scenario Incongruence Report

Scope: compared the current implementation in `src/scenarios/*.rs`, the scenario registry in `src/scenarios/mod.rs`, API-facing data in `src/protocol.rs`, tests in `tests/scenario_runs.rs`, and author docs in `scenarios/*.md` plus `scenarios/README.md`.

Assumption: Rust implementation, protocol data, and tests are the current source of truth; markdown scenario files are documentation/reference material that may be stale.

## Global Findings

1. Registered scenario count does not match markdown docs.
   - `src/scenarios/mod.rs:312-336` registers 22 scenarios.
   - `scenarios/README.md:25-45` lists 21 scenarios and does not include `02-arena-fight-while-dead`.
   - There is no `scenarios/02-arena-fight-while-dead.md`.
   - This also creates duplicate visible numbering: `02-arena-fight-while-dead` and `02-target-validation-range`.

2. Packet schema docs are broader than API-exposed packet lists.
   - `ScenarioSummary` exposes `scenario.packets()` as the player packet list via `src/protocol.rs:156-171`.
   - Most markdown files include `S->C` packets and some unsupported or non-exposed `C->S` packets in "Packet schemas".
   - If markdown packet schemas are intended to document the left panel, they should match `packets()`. If they are author-only protocol notes, rename the section to avoid confusion.

3. Category naming has drifted.
   - Markdown/README categories are bug-class oriented, such as `Target validation`, `Auction house`, `Duplication`.
   - Runtime `category()` values are player/location oriented, such as `Gatehouse`, `Market`, `Post Office`.
   - This may be intentional player-safe wording, but it means docs and API categories are not comparable as written.

## Per-Scenario Findings

### 01 - `01-first-blood-batch`

- Hard mismatch: markdown says the monster has 100 HP (`scenarios/01-first-blood-batch.md:13` and initial state), but the runtime scene shows `Monster #1 (120 HP)` (`src/scenarios/scenario_01_first_blood_batch.rs:22`).
- The intended exploit text says three 40-damage attacks deal 120 damage, so the implementation and exploit agree with 120 HP while the setup text is stale.

### 02 - `02-arena-fight-while-dead`

- Missing documentation: registered at `src/scenarios/mod.rs:314`, implemented in `src/scenarios/scenario_02_arena_fight_while_dead.rs:28-53`, and tested in `tests/scenario_runs.rs:15`, but absent from `scenarios/README.md` and `scenarios/*.md`.
- Current player-facing implementation matches the requested Arena 2 wording: title `Arena 2`, category `Arena`, difficulty `★★☆`, objective `Kill the monster.` (`src/scenarios/scenario_02_arena_fight_while_dead.rs:34-44`).

### 02 - `02-target-validation-range`

- Map/setup mismatch: markdown places lever #7 at `(10, 0)` and describes guard lethal radius 4 (`scenarios/02-target-validation-range.md:30-32`), while the implementation places the lever at `(6, 2)`, gate at `(6, 3)`, and blocked corridor on row `y=3` (`src/scenarios/scenario_02_target_validation_range.rs:26-57`).
- The markdown naive route walks along `y=0`; the visible implementation is a corridor around `y=3`.

### 04 - `04-target-validation-faction`

- No hard incongruence found. The docs and implementation agree on ordering allied cannon #2 to fire at commander #1.

### 05 - `05-auction-negative-price`

- No objective/solution mismatch found.
- Minor schema drift: markdown exposes `CreateListing` (`scenarios/05-auction-negative-price.md:19`), but runtime `packets()` exposes only `BuyListing` (`src/scenarios/scenario_05_auction_negative_price.rs:49-53`).

### 06 - `06-auction-buyout-race`

- Price mismatch: markdown initial listing price is 100 (`scenarios/06-auction-buyout-race.md:30`), but API market data uses price 250 (`src/protocol.rs:460`).
- Minor schema drift: markdown exposes `QueryListing` (`scenarios/06-auction-buyout-race.md:19`), but runtime `packets()` exposes only `Buyout` (`src/scenarios/scenario_06_auction_buyout_race.rs:49-53`).

### 07 - `07-auction-cancel-refund-dupe`

- Hard mismatch: markdown says canceling listing 31 returns the sword and creates mail #2 (`scenarios/07-auction-cancel-refund-dupe.md:48`).
- Runtime solution requires canceling listing 32 (`src/scenarios/scenario_07_auction_cancel_refund_dupe.rs:47-51`), while listing 31 is explicitly a Copper Charm decoy (`src/scenarios/scenario_07_auction_cancel_refund_dupe.rs:92-95`, `src/protocol.rs:474-480`).
- Tests assert canceling listing 31 must not win (`tests/scenario_runs.rs:179-184`, `tests/scenario_runs.rs:412-427`).
- Player-facing setup in markdown says the sword listing is pending and cancelable, but API market data shows the visible cancelable listing is a non-objective Copper Charm and the sword listing is sold with no cancel packet (`tests/scenario_runs.rs:668-688`).

### 08 - `08-dupe-mail-desync`

- No hard incongruence found. Markdown uses `await DraftCreated`, but the engine treats `await` as a no-op placeholder, and the remaining concrete script matches the runtime solution.

### 09 - `09-dupe-trade-window`

- No hard incongruence found.
- Minor title drift: markdown heading is `Split-Second Trade: Trade Window TOCTOU`; implementation title is `Trade Ghost: Remove After Confirm`.

### 10 - `10-dupe-stack-split-negative`

- No hard incongruence found. Markdown asks for at least 99 arrows; implementation checks the negative split plus merge pattern, and the provided solution uses `count: -89`.

### 11 - `11-currency-integer-overflow`

- Hard visible-state mismatch: markdown says the castle deed is priced at 2,000,000,000 gold (`scenarios/11-currency-integer-overflow.md:9`, `scenarios/11-currency-integer-overflow.md:31`), but the runtime scene label says `Deed vendor: price 500` (`src/scenarios/scenario_11_currency_integer_overflow.rs:28`).
- The overflow solution still withdraws `2147483647`, so the visible price of 500 makes the scenario setup look internally inconsistent.

### 12 - `12-toctou-buy-and-use`

- Visible economy mismatch: markdown says antidotes cost 200 and the player has 50 gold (`scenarios/12-toctou-buy-and-use.md:14`, `scenarios/12-toctou-buy-and-use.md:30`).
- Runtime scene says `Apothecary: antidote price 50` (`src/scenarios/scenario_12_toctou_buy_and_use.rs:22`), and API inventory gives the player 1 gold in `APOTHECARY_INVENTORY` (`src/protocol.rs:323-335`).

### 13 - `13-rate-limit-timestamp`

- No hard incongruence found. Markdown uses timestamp `0`; implementation uses `42`, and tests assert any repeated identical timestamp should work.

### 14 - `14-rollback-move-teleport`

- Map geometry mismatch: markdown says wall at `x=5` and treasure at `(8, 0)` (`scenarios/14-rollback-move-teleport.md:13`, `scenarios/14-rollback-move-teleport.md:29-30`).
- Runtime scene has wall tiles around `x=3`, `y=2..4`, and `Relic #77` at `(6, 3)` (`src/scenarios/scenario_14_rollback_move_teleport.rs:13-38`).

### 15 - `15-replay-signed-loot`

- Hard packet/schema mismatch: markdown describes `ClaimLoot` with `signature` and a `LootGrant` packet (`scenarios/15-replay-signed-loot.md:13-21`, `scenarios/15-replay-signed-loot.md:42-48`).
- Runtime packets omit `signature`, and the solution uses fixed `grant_id: 1` (`src/scenarios/scenario_15_replay_signed_loot.rs:56-65`).
- The markdown intended exploit uses `g.grant_id` and `g.signature`; the parser does not substitute captured await values, so the markdown script would not behave like the runtime `solution_script`.

### 16 - `16-cooldown-bypass-batch`

- No hard incongruence found. The visible objective, shield math, packets, and tested solution line up.

### 17 - `17-quest-turnin-double`

- No hard incongruence found.

### 18 - `18-instanced-loot-ownership`

- Reference-script mismatch: markdown intended exploit captures `let d = await LootDrop { owner: 2 }` and sends `Loot { drop: d.drop }` (`scenarios/18-instanced-loot-ownership.md:44-47`).
- Runtime solution uses concrete `Loot { drop: 7002 }` (`src/scenarios/scenario_18_instanced_loot_ownership.rs:56-60`), and the scene reveals `Ally with drop #7002` (`src/scenarios/scenario_18_instanced_loot_ownership.rs:23-28`).
- Because captured await values are not substituted by the parser, the markdown script is not equivalent to the runtime solution.

### 19 - `19-quest-cancel-restart-farm`

- No hard incongruence found.

### 20 - `20-chest-multi-interaction-dupe`

- No hard incongruence found.

### 21 - `21-telehacking-position-spoof`

- Hard map/coordinate mismatch: markdown says the shrine is at `x=100`, `y=0` across bridge `x=40..60` (`scenarios/21-telehacking-position-spoof.md:13`, `scenarios/21-telehacking-position-spoof.md:30`).
- Runtime scene and solution use shrine #91 at `(7, 3)` with chasm tiles at `x=3..5`, `y=3` (`src/scenarios/scenario_21_telehacking_position_spoof.rs:23-44`, `src/scenarios/scenario_21_telehacking_position_spoof.rs:77-81`).
- Markdown intended exploit already uses `(7, 3)`, so only the setup/initial-state text is stale.

### 22 - `22-crafting-clientside-materials`

- Hard schema/mechanics mismatch: markdown describes `CraftItem { recipe, materials: [Material] }` (`scenarios/22-crafting-clientside-materials.md:20-21`), but runtime exposes `CraftItem { recipe: Int, material_count: Int }` (`src/scenarios/scenario_22_crafting_clientside_materials.rs:56-62`).
- Material requirements are inconsistent: markdown says 3 Dragon Scales and 1 Iron Hilt (`scenarios/22-crafting-clientside-materials.md:13`, `scenarios/22-crafting-clientside-materials.md:31-35`), while runtime visible scene says recipe #101 needs 4 scales (`src/scenarios/scenario_22_crafting_clientside_materials.rs:22`) and API inventory labels Dragon Scale as `materials (need 4)` (`src/protocol.rs:385-396`).
- Markdown intended exploits using material arrays are not valid for the current parser/runtime scenario.

## Priority Fix List

1. Bring `scenarios/README.md` and scenario docs into sync with the 23 registered scenarios, especially adding `02-arena-fight-while-dead.md`.
2. Fix the hard stale markdown for scenarios 07, 15, 21, and 22 first; these directly contradict working solution scripts or visible API state.
3. Fix visible numeric/map drift in scenarios 01, 02-target-validation-range, 03, 11, 12, and 14.
4. Decide whether markdown packet schema sections are player-facing packet lists or author protocol notes; then either trim them to `packets()` or rename them.
