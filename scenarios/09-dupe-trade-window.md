# 09 — Split-Second Trade: Trade Window TOCTOU

Category: Duplication
Bug class: Trade-window time-of-check/time-of-use
Difficulty: ★★★

## Objective

Finish with both the rare shield and the trade partner's potion.

## Player-facing setup

A training NPC will accept a trade: your shield for its potion. The trade window
has `OfferItem`, `Ready`, and `Confirm` packets. Normal play swaps the items.

## Packet schemas

```
C->S OfferItem { trade: Int, item: Int }
C->S RemoveItem { trade: Int, item: Int }
C->S Ready { trade: Int, ready: Bool }
C->S Confirm { trade: Int }
S->C TradeState { trade: Int, yours: [Int], theirs: [Int], ready: Bool }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { id: 0, inventory: [Shield:1] }
npc    { id: 2, inventory: [Potion:1] }
trade  { id: 41, state: "open" }
```

## Server rule / hidden bug

`Confirm` snapshots the visible trade contents before checking whether later
same-frame `RemoveItem` packets invalidated readiness. Settlement uses the old
snapshot, but inventory validation uses the final trade window.

## Intended exploit

```
send_batch {
    OfferItem { trade: 41, item: 2001 }
    Ready     { trade: 41, ready: true }
    Confirm   { trade: 41 }
    RemoveItem { trade: 41, item: 2001 }
}
```

The NPC's potion is granted from the confirm snapshot, while the shield remains
with the player because it was removed from the final trade contents.

## Naive failure

Confirming normally swaps the shield for the potion. Removing before confirming
makes the NPC reject the trade.

## Exploit recap

The trade confirmed from one view of the offer while a same-frame removal
changed the live item list.
