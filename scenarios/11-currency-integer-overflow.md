# 11 — Rich Beyond Bounds: Currency Overflow

Category: Economy
Bug class: Int32 overflow on gold
Difficulty: ★★☆

## Objective

Buy the castle deed priced at 2,000,000,000 gold while starting with 100 gold.

## Player-facing setup

The bank supports depositing and withdrawing gold. The packet list shows raw
integer amounts. The center scenario displays a castle deed vendor.

## Packet schemas

```
C->S DepositGold  { amount: Int }
C->S WithdrawGold { amount: Int }
C->S BuyItem      { item: Int }
S->C GoldChanged  { amount: Int }
S->C BankChanged  { amount: Int }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { gold: 100, bank: 0, inventory: [] }
vendor_item { id: 6001, price: 2000000000 }
```

## Server rule / hidden bug

The bank balance is stored in a signed 32-bit integer. A very large withdrawal
wraps the player's visible gold into a high positive value before the purchase
check runs.

## Intended exploit

```
send WithdrawGold { amount: 2147483647 }
send BuyItem { item: 6001 }
```

The flawed arithmetic produces enough apparent gold to buy the castle deed.

## Naive failure

```
send BuyItem { item: 6001 }
```

The vendor rejects the purchase because the player has only 100 gold.

## Exploit recap

A huge gold operation crossed the integer boundary and wrapped into a usable
balance.
