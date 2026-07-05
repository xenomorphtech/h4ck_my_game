# 06 — Last Copy: Auction Buyout Race

Category: Auction house
Bug class: TOCTOU on buyout vs. stock
Difficulty: ★★☆

## Objective

Acquire two identical rare gems from a listing that only has one gem.

## Player-facing setup

The auction house listing shows a stack of one rare gem. A normal purchase buys
it once and removes the listing.

## Packet schemas

```
C->S QueryListing { listing: Int }
C->S Buyout       { listing: Int, quantity: Int }
S->C ListingInfo  { listing: Int, item: Int, quantity: Int, price: Int }
S->C InventoryAdd { item: Int, count: Int }
S->C ListingGone  { listing: Int }
```

## Initial state

```
player  { id: 0, gold: 1000, inventory: [] }
listing { id: 21, item: 700, quantity: 1, price: 100 }
```

## Server rule / hidden bug

The server checks listing availability before settlement, but decrements stock
after sending the inventory reward. Two same-frame `Buyout` packets both observe
quantity = 1 before either decrement is committed.

## Intended exploit

```
batch {
    send Buyout { listing: 21, quantity: 1 }
    send Buyout { listing: 21, quantity: 1 }
}
```

Both buyouts pass the availability check and both grant a gem.

## Naive failure

```
send Buyout { listing: 21, quantity: 1 }
sleep 20
send Buyout { listing: 21, quantity: 1 }
```

The second request sees `ListingGone` and fails.

## Defensive note

Lock the listing row or use an atomic compare-and-decrement. The check and the
mutation must be one transaction with idempotent settlement.
