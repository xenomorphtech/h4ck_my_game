# 05 — Reverse Bid: Negative Auction Price

Category: Auction house
Bug class: Signed/unsigned price conversion
Difficulty: ★★☆

## Objective

Buy the Phoenix Mount, which costs more gold than you have.

## Player-facing setup

The Phoenix Mount is listed for 500 gold. You only have 100. The packet schema
for listing and buying auctions exposes raw `price` values.

## Packet schemas

```
C->S CreateListing { item: Int, price: Int }
C->S BuyListing    { listing: Int, offered_price: Int }
S->C GoldChanged   { amount: Int }
S->C InventoryAdd  { item: Int, count: Int }
```

## Initial state

```
player { id: 0, gold: 100, inventory: [] }
listing { id: 11, item: 9001, seller: 99, price: 500 }
```

## Server rule / hidden bug

The buy path compares `offered_price >= listing.price`, but the settlement path
casts `offered_price` through a different signed representation. A crafted
negative value passes one layer's normalization but credits/charges incorrectly
in another layer.

## Intended exploit

```
send BuyListing { listing: 11, offered_price: -1 }
```

The flawed normalization lets the buy complete while the player's balance is not
properly debited, awarding the mount.

## Naive failure

```
send BuyListing { listing: 11, offered_price: 500 }
```

The server rejects the purchase because the player has only 100 gold.

## Defensive note

Use a single canonical money type, reject negative prices at schema validation,
validate buyer balance and listing price in one transaction, and avoid mixed
signed/unsigned conversions.
