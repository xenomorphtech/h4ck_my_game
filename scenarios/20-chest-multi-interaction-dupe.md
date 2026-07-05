# 20 — Greedy Chest: Multi-Interaction Reward Dupe

Category: Loot
Bug class: Reward granted before "opened" flag commits
Difficulty: ★★☆

## Objective

Collect three reward bundles.

## Player-facing setup

A treasure chest sits in the room. The normal client sends a single `OpenChest`
interaction and greys out the chest afterward. The packet feed shows that the
reward is sent before the chest reports as opened.

## Packet schemas

```
C->S OpenChest { chest: Int }
S->C ChestReward { chest: Int, item: Int, count: Int }
S->C ChestState  { chest: Int, opened: Bool }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { inventory: [] }
chest { id: 81, opened: false, reward: [Gem:3] }
```

## Server rule / hidden bug

The open handler checks `chest.opened == false`, dispatches the reward, and only
then marks the chest opened. Multiple `OpenChest` packets in the same frame each
see `opened == false` and each dispatch the reward.

## Intended exploit

```
batch {
    send OpenChest { chest: 81 }
    send OpenChest { chest: 81 }
    send OpenChest { chest: 81 }
}
```

All three interactions pass the not-yet-opened check and each grants the reward.

## Naive failure

```
send OpenChest { chest: 81 }
sleep 50
send OpenChest { chest: 81 }
```

The second open sees `opened == true` and yields nothing.

## Defensive note

Flip the opened flag atomically before granting rewards (compare-and-set), or
process interactions per-object with a lock. Reward dispatch must be idempotent
against the object's terminal state.
