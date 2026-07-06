# 07 — Sold and Cancelled: Auction Refund Dupe

Category: Auction house
Bug class: Cancel + sell double-settlement
Difficulty: ★★★

## Objective

End the scenario with both the listed sword and the buyer's gold.

## Player-facing setup

You own a sword listed on the auction house. The listing is pending sale and the
UI shows a cancel button. A separate copy of the sale has already settled and its
proceeds are waiting in your mailbox.

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
listing { id: 31, seller: 0, item: 501, price: 300, status: "pending" }
listing { id: 32, seller: 0, item: 501, price: 300, status: "sold" }
mail    { id: 1, attachment: "Gold", status: "unclaimed" }   # sale proceeds
```

## Server rule / hidden bug

Cancel and sale are settled through separate records against the same item.
Canceling the pending listing enqueues a returned-item mail (#2) even though the
sale already settled and paid gold (mail #1). Both settlement paths pay out, so
the seller keeps the sword and the gold. Timing does not matter: the cancel path
does not check whether the item was already sold.

## Intended exploit

```
send CancelListing { listing: 31 }   # returns the sword -> creates mail #2
send ClaimMailbox { mail: 1 }        # gold from the completed sale
send ClaimMailbox { mail: 2 }        # the returned sword
```

The player claims both rewards: sword returned + sale gold. The cancel can be
sent at any time (manually via the button or scripted); mail #2 only exists after
the cancel is sent.

## Naive failure

Claiming mailbox rewards without ever canceling: mail #2 never exists, so only
the sale gold is received and the sword stays sold.

## Defensive note

A listing needs a single terminal state machine (`active -> sold` or
`active -> cancelled`, never both), with a uniqueness constraint on settlement
records and idempotent mailbox rewards.
