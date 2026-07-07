# 07 — Sold and Cancelled: Auction Refund Dupe

Category: Auction house
Bug class: Cancel + sell double-settlement
Difficulty: ★★★

## Objective

End the scenario with both the listed sword and the buyer's gold.

## Player-facing setup

You have a visible Copper Charm listing that can be cancelled from the market
panel. Your sword sale has already settled, and its proceeds are waiting in your
mailbox.

## Packet schemas

```
C->S CancelListing { listing: Int }
C->S ClaimMailbox  { mail: Int }
S->C ListingSold   { listing: Int, mail: Int }
S->C ListingCancel { listing: Int, mail: Int }
S->C MailClaimed   { mail: Int }
S->C MailClaimFailed { mail: Int, reason: String }
S->C InventoryAdd  { item: Int, count: Int }
S->C GoldChanged   { amount: Int }
```

## Initial state

```
player  { id: 0, gold: 0, inventory: [] }
listing { id: 31, seller: 0, item: "Copper Charm", price: 120, status: "pending" }
listing { id: 32, seller: 0, item: "Listed Sword", price: 300, status: "sold" }
mail    { id: 1, attachment: "Gold", status: "unclaimed" }   # sale proceeds
```

## Server rule / hidden bug

Cancel and sale are settled through separate records against the same item. The
sold sword listing can still be cancelled by raw packet, which enqueues the next
mailbox reward even though the sale already settled and paid gold (mail #1).
Both settlement paths pay out, so the seller keeps the sword and the gold. Timing
does not matter: the cancel path does not check whether the item was already
sold.

## Intended exploit

```
send CancelListing { listing: 32 }   # returns the sword -> creates mail #2
send ClaimMailbox { mail: 1 }        # gold from the completed sale
send ClaimMailbox { mail: 2 }        # the returned sword
```

The player claims both rewards: sword returned + sale gold. If another listing is
cancelled first, it consumes mail #2 and the sword return receives the next mail
ID.

## Naive failure

```
send ClaimMailbox { mail: 1 }
```

Claiming the sale proceeds alone receives only gold; the sword stays sold.

## Exploit recap

The sale payout and cancel refund both landed, leaving you with the gold and the
returned item.
