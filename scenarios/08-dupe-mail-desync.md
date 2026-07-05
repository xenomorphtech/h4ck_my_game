# 08 — Phantom Attachment: Mail Desync Dupe

Category: Duplication
Bug class: Attach/withdraw acknowledgement desync
Difficulty: ★★☆

## Objective

End with two Dragon Scales. You start with one.

## Player-facing setup

The mailbox supports attaching an item to a draft, sending it, and cancelling
the draft. The packet feed shows separate acknowledgements for `AttachItem` and
`CancelDraft`.

## Packet schemas

```
C->S CreateDraft { recipient: Int }
C->S AttachItem  { draft: Int, item: Int }
C->S CancelDraft { draft: Int }
C->S ClaimMail   { mail: Int }
S->C DraftCreated { draft: Int }
S->C AttachmentAck { draft: Int, item: Int }
S->C InventoryAdd { item: Int, count: Int }
```

## Initial state

```
player { id: 0, inventory: [DragonScale:1] }
mailbox { drafts: [], mail: [] }
```

## Server rule / hidden bug

`AttachItem` removes the item from inventory after an async acknowledgement.
`CancelDraft` returns all draft attachments immediately. If cancel is batched
with attach, the draft return path can add the item while the delayed removal is
suppressed because the draft no longer exists.

## Intended exploit

```
send CreateDraft { recipient: 0 }
await DraftCreated { draft: 1 }

batch {
    send AttachItem  { draft: 1, item: 1001 }
    send CancelDraft { draft: 1 }
}
```

The inventory keeps the original Dragon Scale and receives the cancelled draft's
returned copy.

## Naive failure

Attach, wait, then cancel: the item is removed first and simply returned once.

## Defensive note

Inventory transfer must be atomic: remove from source and add to destination in
one transaction. Cancelling a draft should reference committed attachment rows,
not in-flight client intent.
