# 15 — Echo Chest: Replay Signed Loot

Category: Replay
Bug class: No nonce on signed packet
Difficulty: ★★★

## Objective

Claim three relics from the chest.

## Player-facing setup

Opening the chest produces a `LootGrant` packet in the feed. The packet includes
a `signature`, making it look authoritative.

## Packet schemas

```
C->S OpenChest { chest: Int }
C->S ClaimLoot { chest: Int, grant_id: Int, signature: String }
S->C LootGrant { chest: Int, grant_id: Int, item: Int, signature: String }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { id: 0, inventory: [] }
chest { id: 51, opened: false, item: Relic }
```

## Server rule / hidden bug

The signature proves the grant came from the server, but the claim endpoint does
not track whether a `grant_id` has already been consumed. Replaying a captured
`ClaimLoot` with the same signature grants the item again.

## Intended exploit

```
send OpenChest { chest: 51 }
let g = await LootGrant { chest: 51 }

for i in 1..4 {
    send ClaimLoot {
        chest: 51,
        grant_id: g.grant_id,
        signature: g.signature,
    }
}
```

The same signed grant is redeemed three times.

## Naive failure

Opening the chest normally gives exactly one relic.

## Defensive note

Signed payloads still need nonces or unique grant IDs with consumed-state
tracking. Claims must be idempotent: replaying a valid old token should not
create new rewards.
