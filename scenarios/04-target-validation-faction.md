# 04 — Friendly Fire: Missing Faction Gate

Category: Target validation
Bug class: Missing ownership/faction gate
Difficulty: ★★☆

## Objective

Stop the enemy commander.

## Player-facing setup

The enemy commander has an invulnerability aura. Allied cannons can pierce the
aura, but the UI only lets you order your own player to attack. The packet list
reveals an `OrderUnit` command used by friendly NPCs.

## Packet schemas

```
C->S Attack    { target: Int }
C->S OrderUnit { unit: Int, action: String, target: Int }
S->C Damage    { source: Int, target: Int, amount: Int }
S->C Death     { id: Int }
```

## Initial state

```
player     { id: 0, faction: "ally" }
commander  { id: 1, hp: 100, aura: "player_damage_immune" }
allyCannon { id: 2, faction: "ally", damage: 100 }
```

## Server rule / hidden bug

`OrderUnit` checks that the ordered unit exists, but not that the requesting
player owns or commands that unit. Any friendly NPC can be instructed.

## Intended exploit

```
send OrderUnit { unit: 2, action: "fire", target: 1 }
```

The allied cannon kills the commander, bypassing the player's damage immunity
restriction without harming allies.

## Naive failure

```
send Attack { target: 1 }
```

Player attacks are absorbed by the aura forever.

## Defensive note

Server commands must validate actor authority: ownership, faction permissions,
AI commandability, encounter state, and command type.
