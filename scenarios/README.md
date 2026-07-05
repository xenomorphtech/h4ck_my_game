# Scenarios

One file per puzzle. Every scenario is a simulated game-server bug you must
exploit with a Netscript solution to complete the objective. Normal play is
always insufficient — you must abuse the server rule described in each file.

Each scenario file follows the same structure:

- Category & difficulty
- Objective (shown in the in-game banner)
- Packet schemas (what the left panel exposes)
- Initial state
- Server rules (the bug — normally hidden from the player)
- Intended exploit (author reference solution)
- Naive attempt (why fair play fails)
- Defensive note (the real-world fix — this is a training game)

All scenarios are deterministic: seeded PRNG, virtual clock, no real I/O.
See `../SPEC.md` for the engine, UI, and Netscript language reference.

## Index

| # | File | Category | Bug class | Difficulty |
|---|------|----------|-----------|------------|
| 01 | `01-first-blood-batch.md` | Combat | Per-hit retaliation scheduling | ★☆☆ |
| 02 | `02-target-validation-range.md` | Target validation | Server trusts client range check | ★☆☆ |
| 03 | `03-target-validation-dead.md` | Target validation | Stale target reference / no liveness check | ★☆☆ |
| 04 | `04-target-validation-faction.md` | Target validation | Missing ownership/faction gate | ★★☆ |
| 05 | `05-auction-negative-price.md` | Auction house | Unsigned/underflow on price field | ★★☆ |
| 06 | `06-auction-buyout-race.md` | Auction house | TOCTOU on buyout vs. stock | ★★☆ |
| 07 | `07-auction-cancel-refund-dupe.md` | Auction house | Cancel + sell double-settlement | ★★★ |
| 08 | `08-dupe-mail-desync.md` | Duplication | Attach/withdraw ack desync | ★★☆ |
| 09 | `09-dupe-trade-window.md` | Duplication | Trade-window TOCTOU | ★★★ |
| 10 | `10-dupe-stack-split-negative.md` | Duplication | Negative quantity stack split | ★★☆ |
| 11 | `11-currency-integer-overflow.md` | Economy | Int32 overflow on gold | ★★☆ |
| 12 | `12-toctou-buy-and-use.md` | Race | Use-before-debit ordering | ★★☆ |
| 13 | `13-rate-limit-timestamp.md` | Rate limiting | Dedupe by timestamp only | ★★☆ |
| 14 | `14-rollback-move-teleport.md` | Movement | Client-authoritative position | ★★★ |
| 15 | `15-replay-signed-loot.md` | Replay | No nonce on signed packet | ★★★ |
| 16 | `16-cooldown-bypass-batch.md` | Combat | Cooldown checked after apply | ★★☆ |
| 17 | `17-quest-turnin-double.md` | Progression | Idempotency missing on turn-in | ★★☆ |
| 18 | `18-instanced-loot-ownership.md` | Loot | Loot ownership not enforced | ★★★ |
| 19 | `19-quest-cancel-restart-farm.md` | Progression | Quest cancel/restart starter item farm | ★★☆ |
| 20 | `20-chest-multi-interaction-dupe.md` | Loot | Multiple interactions before opened flag commits | ★★☆ |
| 21 | `21-telehacking-position-spoof.md` | Movement | Client position spoof for remote interaction | ★★★ |
| 22 | `22-crafting-clientside-materials.md` | Crafting | Client-side material list / quantity tampering | ★★★ |

★☆☆ intro · ★★☆ core · ★★★ advanced
