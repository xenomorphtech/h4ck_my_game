# 16 — No Cooldown Yet: Batched Skill Spam

Category: Combat
Bug class: Cooldown checked after apply
Difficulty: ★★☆

## Objective

Break the boss shield.

## Player-facing setup

`PowerStrike` deals 50 shield damage but has a 1 second cooldown. The boss
shield has 150 HP. Normal play cannot fit three casts before enrage.

## Packet schemas

```
C->S CastSkill { skill: Int, target: Int }
S->C SkillResult { skill: Int, target: Int, amount: Int }
S->C CooldownStarted { skill: Int, ready_at_ms: Int }
S->C ShieldBroken { target: Int }
```

## Initial state

```
player { id: 0, cooldowns: {} }
boss { id: 1, shield_hp: 150, enrage_at_ms: 100 }
PowerStrike { skill: 10, damage: 50, cooldown_ms: 1000 }
```

## Server rule / hidden bug

The server checks cooldown at the beginning of each packet, applies skill damage,
and only writes the new cooldown after the entire input frame completes. Three
same-frame casts all see the skill as ready.

## Intended exploit

```
batch {
    send CastSkill { skill: 10, target: 1 }
    send CastSkill { skill: 10, target: 1 }
    send CastSkill { skill: 10, target: 1 }
}
```

All three hits land before the cooldown state is committed.

## Naive failure

```
send CastSkill { skill: 10, target: 1 }
sleep 1000
send CastSkill { skill: 10, target: 1 }
```

The boss enrages at t=100ms before the second cast.

## Defensive note

Cooldowns should be reserved before effect application, or action processing
should use per-action atomic state updates instead of end-of-frame commits.
