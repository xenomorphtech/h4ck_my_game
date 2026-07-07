# Puzzle audit tasklist

## Cross-cutting

- [x] Keep spoiler safety as semantic review; do not re-add deterministic forbidden-word tests.
- [x] Add/keep behavioral and structural tests for puzzle logic, visible affordances, and reveal gates.
- [x] Make important packet ids discoverable from visible labels, packet feed, or neutral packet schemas.
- [x] Ensure scene obstacles match stated fiction with `blocked_tiles` where relevant.
- [x] Remove or relabel misleading floor item entities that are filtered from scene rendering.
- [ ] Reconcile docs with shipped mechanics after scenario fixes.

## Scenario tasks

### 01 — `01-first-blood-batch`
- [x] Reconcile monster HP docs/client/lesson around 120 HP.
- [x] Keep objective as visible goal only.
- [x] Add/keep behavioral checks that spaced attacks lose and same-tick attacks win.

### 02 Arena — `02-arena-fight-while-dead`
- [x] Keep objective as `Kill the monster.` with no death/action-order hint.
- [x] Remove client death-overlay text that leaks accepted attack packets after death.
- [x] Make Arena 2 visually/mechanically distinct from Arena 1 enough to justify different behavior.
- [x] Add/keep behavioral checks for required pre-death hit + post-death burst and Arena 1 batch losing here.

### 02 Gatehouse — `02-target-validation-range`
- [x] Add a visible lever/interactable for `Use { target: 7 }`.
- [x] Make the guard block a real corridor with blocked wall/guard tiles.
- [x] Make guard/agro premise visible without explaining the range-validation bug.
- [x] Replace misleading Gate Key inventory if the puzzle is lever-based.
- [x] Fix objective plural/singular if needed.

### 04 — `04-target-validation-faction`
- [x] Make cannon/commander ids and allowed order action discoverable without exposing the bug.
- [x] Add visible shield/aura or copy cue explaining direct attacks fail.
- [x] Reconcile siege inventory/theme.

### 05 — `05-auction-negative-price`
- [x] Keep current setup; no blocking fixes.
- [ ] Optionally tighten win condition around acquisition semantics.

### 06 — `06-auction-buyout-race`
- [x] Give enough gold to afford two gems so stock is the visible blocker.
- [ ] Reconcile docs with inventory gold.
- [x] Add/keep behavioral check that one bulk quantity does not win.

### 07 — `07-auction-cancel-refund-dupe`
- [x] Make scheduled sale timing and mailbox ids discoverable without revealing the race.
- [ ] Add a buyer or feed cue if needed.
- [ ] Add/keep wrong-timing behavior check.

### 08 — `08-dupe-mail-desync`
- [ ] Add a normal mail flow (`SendDraft`) or remove the dead `ClaimMail` affordance.
- [x] Remove/relabel loose scale that implies pickup.
- [ ] Tighten win condition to the intended item where useful.

### 09 — `09-dupe-trade-window`
- [x] Put packet schemas in natural trade order.
- [ ] Decide and document whether within-tick order matters.
- [ ] Add/keep behavior check that remove outside the same tick loses.

### 10 — `10-dupe-stack-split-negative`
- [x] Remove invisible loose arrow scene entity or replace it with a visible prop.
- [x] Add empty destination slot to inventory.
- [x] Tighten merge args if useful.
- [ ] Reconcile docs/objective.

### 11 — `11-currency-integer-overflow`
- [x] Make deed/vendor and price visible.
- [ ] Consider neutral objective once price is visible.
- [ ] Add deposit/bank affordance or visible bank balance.
- [ ] Reconcile docs/packets.

### 12 — `12-toctou-buy-and-use`
- [ ] Reconcile visible gold amount with docs.
- [x] Surface antidote price visibly.
- [x] Remove/replace invisible potion scene entity.
- [x] Optionally add visible poison hazard cue.

### 13 — `13-rate-limit-timestamp`
- [x] Fix win logic to accept repeated identical non-zero client timestamps.
- [x] Make increasing timestamps lose.
- [x] Sync naive script/docs with distinct timestamp behavior.

### 14 — `14-rollback-move-teleport`
- [x] Add blocked wall tiles matching the objective.
- [x] Make the wall a real barrier/corridor.
- [ ] Give scenario-specific ruins/relic inventory.
- [x] Make target id discoverable or keep consistent with labels/docs.

### 15 — `15-replay-signed-loot`
- [ ] Implement or expose a grant/signature-like discovery surface, or reconcile docs to current grant-id replay.
- [x] Make grant id discoverable.
- [x] Remove/relabel loose relic if it contradicts chest source.

### 16 — `16-cooldown-bypass-batch`
- [x] Make scene template/category/inventory coherent.
- [x] Add a visible skill source and id affordance.
- [x] Add shield visual/status cue.
- [x] Add/keep behavioral check that spaced casts lose.

### 17 — `17-quest-turnin-double`
- [ ] Reconcile category with docs.
- [x] Align quest item fiction and inventory sprite/name.
- [ ] Decide whether 4 or 5 abandons should be required.

### 18 — `18-instanced-loot-ownership`
- [x] Make required drop id discoverable via schema/feed/visible label.
- [x] Add visible party member owner.
- [x] Render/label the owned drop clearly.
- [ ] Reconcile docs with static vs feed-based implementation.

### 19 — `19-quest-cancel-restart-farm`
- [x] Surface that accepting Supply Run grants a kit.
- [ ] Reconcile category with docs.
- [ ] Decide abandon count semantics.
- [x] Relabel kit prop if it implies pickup.

### 20 — `20-chest-multi-interaction-dupe`
- [x] Make naive/example three sequential opens so timing, not count, is the missing ingredient.
- [x] Optionally show reward target as 0/3.
- [x] Add/keep timing-not-count behavior check.

### 21 — `21-telehacking-position-spoof`
- [x] Align visible shrine coordinates with `check_win` coordinates.
- [x] Reconcile docs/scene/scripts to one coordinate space.
- [x] Consider validating `client_y`.
- [x] Make target id discoverable.

### 22 — `22-crafting-clientside-materials`
- [ ] Reconcile docs with scalar `material_count` implementation or implement material-list version.
- [x] Accept conceptually correct under-declared material counts if keeping scalar model.
- [x] Surface recipe requirement visibly.
- [ ] Reconsider difficulty after mechanics are coherent.
