# 16 — No Cooldown Yet: Batched Skill Spam

Category: Combat
Bug class: Cooldown checked after apply
Difficulty: ★★☆

## Objective

Break the boss shield and finish the 1 HP boss before it retaliates. Shield: 150. PowerStrike: 50 shield damage, 1000ms cooldown. Attack: 10 HP damage, 750ms cooldown. Retaliation: 500ms.

## Player-facing setup

`PowerStrike` deals 50 shield damage and has a 1000ms cooldown. `Attack`
deals 10 HP damage and has a 750ms cooldown. The boss has 1 HP behind a
150 durability shield and retaliates after 500ms.

## Packet schemas

```
C->S CastSkill { skill: Int, target: Int }
C->S Attack { target: Int }
S->C SkillResult { skill: Int, target: Int, amount: Int }
S->C CooldownStarted { skill: Int, ready_at_ms: Int }
S->C ShieldBroken { target: Int }
```

## Initial state

```
player { id: 0, cooldowns: {} }
boss { id: 1, hp: 1, shield_hp: 150, enrage_at_ms: 500 }
Attack { damage: 10, cooldown_ms: 750 }
PowerStrike { skill: 10, shield_damage: 50, cooldown_ms: 1000 }
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
    send Attack { target: 1 }
}
```

All three PowerStrikes and the finishing Attack land in the same frame,
before the cooldown state is committed and before the boss retaliates.

## Naive failure

```
send CastSkill { skill: 10, target: 1 }
sleep 1000
send CastSkill { skill: 10, target: 1 }
```

The boss retaliates at t=500ms before the second cast, and the shield is
never fully broken.

## Defensive note

Cooldowns should be reserved before effect application, or action processing
should use per-action atomic state updates instead of end-of-frame commits.
