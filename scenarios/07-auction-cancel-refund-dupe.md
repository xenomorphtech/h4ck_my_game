# 07 — Sold and Cancelled: Auction Refund Dupe

Category: Auction house
Bug class: Cancel + sell double-settlement
Difficulty: ★★★

## Objective

End the scenario with both the listed sword and the buyer's gold.

## Player-facing setup

You own a sword listed on the auction house. A scripted buyer will buy it at
t=300ms. The UI allows canceling the listing before it sells.

## Packet schemas

```
C->S CancelListing { listing: Int }
C->S ClaimMailbox  { mail: Int }
S->C ListingSold   { listing: Int, mail: Int }
S->C ListingCancel { listing: Int, mail: Int }
S->C InventoryAdd  { item: Int, count: Int }
S->C GoldChanged   { amount: Int }
```

## Initial state

```
player  { id: 0, gold: 0, inventory: [] }
listing { id: 31, seller: 0, item: 501, price: 1000, status: "active" }
buyer_bot_buy_time: 300ms
```

## Server rule / hidden bug

Cancel and sale are separate scheduled jobs. If `CancelListing` lands in the
same frame as buyer settlement, both paths enqueue mailbox rewards: cancellation
returns the item, sale sends the gold.

## Intended exploit

```
at(300) { send CancelListing { listing: 31 } }
await ListingSold { listing: 31 }
await ListingCancel { listing: 31 }

send ClaimMailbox { mail: 1 }
send ClaimMailbox { mail: 2 }
```

The player claims both rewards: sword returned + sale gold.

## Naive failure

Cancel too early: only the sword is returned. Cancel too late: only gold is
received.

## Defensive note

A listing needs a single terminal state machine (`active -> sold` or
`active -> cancelled`, never both), with a uniqueness constraint on settlement
records and idempotent mailbox rewards.
