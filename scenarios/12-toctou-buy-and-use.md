# 12 — Drink Before Paying: Buy-and-Use TOCTOU

Category: Race
Bug class: Use-before-debit ordering
Difficulty: ★★☆

## Objective

Survive the poison cloud without enough gold to buy an antidote.

## Player-facing setup

A poison cloud kills the player at t=500ms unless the player has used an
antidote. The shop sells antidotes for 200 gold, but you have 50.

## Packet schemas

```
C->S BuyItem { item: Int }
C->S UseItem { item: Int }
S->C InventoryAdd { item: Int, count: Int }
S->C StatusEffect { id: Int, active: Bool }
S->C Death { id: Int }
```

## Initial state

```
player { id: 0, gold: 50, poisoned: true, inventory: [] }
antidote { item: 301, price: 200 }
poison_kill_time: 500ms
```

## Server rule / hidden bug

The shop grants the item immediately, then schedules the gold debit/check later
in the same server frame. If `UseItem` arrives in the same batch, it can consume
the antidote before the failed debit rolls back the purchase.

## Intended exploit

```
batch {
    send BuyItem { item: 301 }
    send UseItem { item: 301 }
}
```

The antidote clears poison before the server rejects or rolls back the purchase.

## Naive failure

```
send BuyItem { item: 301 }
```

The purchase fails due to insufficient gold; the poison kills the player.

## Defensive note

Never grant usable inventory before payment succeeds. Process payment and item
grant atomically, or make uncommitted items unusable until settlement completes.
