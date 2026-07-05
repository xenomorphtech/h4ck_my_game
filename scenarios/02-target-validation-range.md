# 02 — Long Arm: Range Validation Bypass

Category: Target validation
Bug class: Server trusts client range check
Difficulty: ★☆☆

## Objective

Open the locked gate. The guards patrol its approach.

## Player-facing setup

A lever controls the gate, but it is placed behind a guard. The normal client
only enables the `Use` button when you stand within 2 tiles of the lever. If you
walk close enough, the guard kills you.

## Packet schemas

```
C->S Move { x: Int, y: Int }
C->S Use  { target: Int }
S->C GateState { id: Int, open: Bool }
S->C Death { id: Int }
```

## Initial state

```
player { id: 0, hp: 100, x: 0, y: 0 }
lever  { id: 7, x: 10, y: 0 }
gate   { id: 8, open: false }
guard  { id: 9, lethal_radius: 4 }
```

## Server rule / hidden bug

The client UI checks use range, but the server only checks that the `target`
exists and is usable. It never validates player distance.

## Intended exploit

```
send Use { target: 7 }
```

The lever activates from across the room and opens the gate.

## Naive failure

```
for x in 1..10 {
    send Move { x: x, y: 0 }
    sleep 50
}
send Use { target: 7 }
```

The guard kills the player when the player enters its lethal radius.

## Defensive note

All authority checks must happen server-side: distance, line of sight, target
state, permissions, and cooldowns. Client UI checks are convenience only.
