# 01 — First Blood: Batch the Fatal Window

Category: Combat
Bug class: Per-hit retaliation scheduling
Difficulty: ★☆☆

## Objective

Kill the monster.

## Player-facing setup

A monster stands in the arena with 100 HP. Your normal attack does 40 damage.
The packet feed shows that every attack causes the server to schedule a lethal
counterattack after 250ms.

## Packet schemas

```
C->S Attack { target: Int, power: Int = 40 }
S->C Damage { source: Int, target: Int, amount: Int }
S->C Death  { id: Int }
```

## Initial state

```
player  { id: 0, hp: 100 }
monster { id: 1, hp: 100 }
```

## Server rule / hidden bug

The server applies incoming attack damage immediately, but schedules retaliation
for +250ms instead of resolving retaliation before additional same-frame attacks.
If three attacks arrive in the same simulation tick, all three damage events
apply before any retaliation can fire.

## Intended exploit

```
send_batch {
    Attack { target: 1 }
    Attack { target: 1 }
    Attack { target: 1 }
}
```

The monster takes 120 damage at t=0 and dies before the scheduled retaliations
at t=250 can matter.

## Naive failure

```
for i in 1..4 {
    send Attack { target: 1 }
    sleep 100
}
```

The first counterattack kills the player at t=250 before enough spaced attacks
land.

## Exploit recap

Three attacks landed in the same packet frame, collapsing the retaliation window
before the monster could answer.
