# 03 — Ghost Target: Stale Reference Attack

Category: Target validation
Bug class: Stale target reference / no liveness check
Difficulty: ★☆☆

## Objective

Defeat the necromancer shielded by four skeletons.

## Player-facing setup

The necromancer is immune while any skeleton is alive. When a skeleton dies, the
server emits a `TargetHint` packet pointing at the necromancer for a few frames.
Normal attacks against the necromancer fail while the shield is active.

## Packet schemas

```
C->S Attack { target: Int }
S->C Damage { target: Int, amount: Int }
S->C Death { id: Int }
S->C TargetHint { id: Int, valid_until_ms: Int }
```

## Initial state

```
player      { id: 0, hp: 100 }
necromancer { id: 1, hp: 60, shielded: true }
skeletons   { ids: [2,3,4,5], hp: 20 each }
```

## Server rule / hidden bug

The server validates that the client has a recent `TargetHint`, but does not
re-check the target's current shield/liveness state when resolving a batched
attack packet.

## Intended exploit

```
send Attack { target: 2 }
await TargetHint { id: 1 }

batch {
    send Attack { target: 1 }
    send Attack { target: 1 }
    send Attack { target: 1 }
}
```

The stale hint lets attacks resolve directly against the shielded necromancer.

## Naive failure

Killing all skeletons normally gives the necromancer enough time to cast a wipe.
Attacking the necromancer without a hint is rejected.

## Defensive note

Treat hints as presentation data, not authorization. Re-check target liveness,
shield state, and combat legality at packet resolution time.
