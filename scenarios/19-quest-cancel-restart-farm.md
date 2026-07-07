# 19 — Quest Yo-Yo: Cancel/Restart Item Farm

Category: Progression
Bug class: Non-idempotent quest state / grant-on-accept
Difficulty: ★★☆

## Objective

Collect five provision kits.

## Player-facing setup

The quartermaster quest "Supply Run" gives you a Provision Kit when you accept
it, so you can deliver it elsewhere. The UI lets you abandon the quest, but hides
the fact that abandoning does not reclaim the starter item.

## Packet schemas

```
C->S AcceptQuest  { quest: Int }
C->S AbandonQuest { quest: Int }
S->C QuestState   { quest: Int, status: String }
S->C InventoryAdd    { item: Int, count: Int }
S->C InventoryRemove { item: Int, count: Int }
```

## Initial state

```
player { inventory: [], active_quests: [] }
quest { id: 71, name: "Supply Run", starter_item: ProvisionKit }
```

## Server rule / hidden bug

Accepting the quest grants the starter item immediately. Abandoning the quest
resets its status to "available" but never removes the granted starter item.
Accept -> abandon can be repeated to accumulate items.

## Intended exploit

```
for i in 1..6 {
    send AcceptQuest  { quest: 71 }
    send AbandonQuest { quest: 71 }
}
```

Each accept grants a Provision Kit; each abandon frees the quest to be accepted
again, leaving all granted kits in inventory.

## Naive failure

Accepting once and completing the quest normally yields a single reward and locks
the quest as completed.

## Exploit recap

Quest restart state granted starter kits repeatedly while previous grants
remained in inventory.
