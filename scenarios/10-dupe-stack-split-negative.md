# 10 — Minus One Stack: Negative Split Dupe

Category: Duplication
Bug class: Negative quantity stack split
Difficulty: ★★☆

## Objective

Create at least 99 arrows from a stack of 10.

## Player-facing setup

The inventory lets you split item stacks. The normal UI only allows positive
split counts between 1 and stack size - 1.

## Packet schemas

```
C->S SplitStack { slot: Int, count: Int }
C->S MergeStack { from_slot: Int, to_slot: Int }
S->C SlotChanged { slot: Int, item: Int, count: Int }
```

## Initial state

```
inventory {
    slot 0: { item: Arrow, count: 10 }
    slot 1: empty
}
```

## Server rule / hidden bug

The server validates `count < source.count`, but forgets `count > 0`. Splitting
`-89` subtracts a negative number from the source stack and creates a malformed
negative stack in the destination. Merging normalizes the destination by adding
absolute quantities.

## Intended exploit

```
send SplitStack { slot: 0, count: -89 }
send MergeStack { from_slot: 1, to_slot: 0 }
```

The source stack grows because subtracting a negative count adds arrows.

## Naive failure

```
send SplitStack { slot: 0, count: 9 }
```

This simply creates stacks of 1 and 9; no duplication.

## Defensive note

Validate all quantity fields as bounded positive integers at packet decoding.
Represent inventory counts with non-negative domain types, not raw signed ints.
