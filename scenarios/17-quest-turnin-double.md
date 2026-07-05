# 17 — Twice for One Head: Double Quest Turn-In

Category: Progression
Bug class: Missing idempotency on quest completion
Difficulty: ★★☆

## Objective

Earn two reward chests from a quest that should pay once.

## Player-facing setup

The quest requires one wolf head. You have exactly one. The normal UI disables
`TurnInQuest` after completion, but the packet list exposes the raw command.

## Packet schemas

```
C->S TurnInQuest { quest: Int, item: Int }
S->C QuestComplete { quest: Int }
S->C InventoryRemove { item: Int, count: Int }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { inventory: [WolfHead:1], completed_quests: [] }
quest { id: 61, required_item: WolfHead, reward: RewardChest }
```

## Server rule / hidden bug

The server checks for the required item and queues the reward before marking the
quest complete. Two same-frame turn-ins both pass the item/completion checks.
Item removal is deduplicated later, but reward grants are not.

## Intended exploit

```
batch {
    send TurnInQuest { quest: 61, item: 4001 }
    send TurnInQuest { quest: 61, item: 4001 }
}
```

Both reward grants are queued before quest completion is committed.

## Naive failure

Turn in once: one reward chest and the quest becomes completed.

## Defensive note

Quest completion should be idempotent. Mark completion or reserve the required
item transactionally before queuing rewards, and enforce unique reward grants per
player+quest.
