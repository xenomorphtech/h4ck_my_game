# 18 — Someone Else's Sparkle: Loot Ownership Bypass

Category: Loot
Bug class: Loot ownership not enforced
Difficulty: ★★★

## Objective

Take another party member's drop.

## Player-facing setup

After the boss dies, a loot sparkle appears. The normal client only shows the
`Loot` button to the owner. The packet feed reveals `LootDrop` packets for all
party members, including owner IDs.

## Packet schemas

```
C->S Attack { target: Int }
C->S Loot   { drop: Int }
S->C LootDrop { drop: Int, owner: Int, item: Int }
S->C InventoryAdd { item: Int, count: Int }
S->C Error { code: String }
```

## Initial state

```
player { id: 0, inventory: [] }
party_member { id: 2 }
boss { id: 1, hp: 1 }
drop_rule: boss drop owner = party_member
```

## Server rule / hidden bug

The server sends loot-drop metadata to all party members and checks only that the
requested drop exists. It does not verify that the requester is the owner or
eligible looter.

## Intended exploit

```
send Attack { target: 1 }
let d = await LootDrop { owner: 2 }
send Loot { drop: d.drop }
```

The player claims a drop assigned to another party member.

## Naive failure

Waiting for the UI never works because the loot button is hidden from non-owners.

## Exploit recap

The loot claim path accepted a drop you could see, even though it belonged to
another player.
