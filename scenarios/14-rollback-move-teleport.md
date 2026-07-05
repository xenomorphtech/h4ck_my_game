# 14 — Rubber Band Launch: Movement Rollback Teleport

Category: Movement
Bug class: Client-authoritative position / rollback abuse
Difficulty: ★★★

## Objective

Reach the treasure room behind the locked wall.

## Player-facing setup

The map has a wall at x=5. The normal client cannot pass it. The server sends
position acknowledgements after movement packets.

## Packet schemas

```
C->S MoveIntent { seq: Int, dx: Int, dy: Int }
C->S Interact   { target: Int }
S->C PositionAck { seq: Int, x: Int, y: Int }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { x: 0, y: 0, last_acked_seq: 0 }
wall { x: 5, blocks: true }
treasure { id: 77, x: 8, y: 0 }
```

## Server rule / hidden bug

Collision checks run against the last acknowledged position, but movement
integration uses the client-submitted sequence order. By sending several moves
before any acknowledgement, the server accumulates displacement from stale valid
positions and only rolls back visually after interaction has already resolved.

## Intended exploit

```
batch {
    send MoveIntent { seq: 1, dx: 2, dy: 0 }
    send MoveIntent { seq: 2, dx: 2, dy: 0 }
    send MoveIntent { seq: 3, dx: 2, dy: 0 }
    send MoveIntent { seq: 4, dx: 2, dy: 0 }
    send Interact { target: 77 }
}
```

The interaction resolves at the temporarily advanced position beyond the wall.

## Naive failure

Walking normally waits for acknowledgements and gets blocked at the wall.

## Defensive note

Authoritative server movement must validate the full path and final position
against current state before applying interactions. Do not let speculative
positions authorize actions.
