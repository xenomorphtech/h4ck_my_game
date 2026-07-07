# 22 — Recipe Is Just a Suggestion: Client-Side Crafting Materials

Category: Crafting
Bug class: Client-authoritative recipe/material list
Difficulty: ★★★

## Objective

Forge the Dragon Blade at the anvil.

## Player-facing setup

The crafting UI shows Dragon Blade requires 3 Dragon Scales and 1 Iron Hilt.
The packet schema reveals that `CraftItem` sends the full material list and
quantities from the client.

## Packet schemas

```
C->S CraftItem { recipe: Int, materials: [Material] }
Material { item: Int, count: Int }
S->C InventoryRemove { item: Int, count: Int }
S->C InventoryAdd    { item: Int, count: Int }
S->C CraftResult     { recipe: Int, ok: Bool }
```

## Initial state

```
player {
    inventory: [Pebble:99, IronHilt:1]
}
recipe DragonBlade {
    id: 101,
    required: [DragonScale:3, IronHilt:1],
    output: DragonBlade
}
```

## Server rule / hidden bug

The server validates that every material listed in the packet exists in the
player inventory, but it does not look up the authoritative recipe requirements.
It trusts the client-provided material list and output recipe ID.

## Intended exploits

Use no materials:

```
send CraftItem { recipe: 101, materials: [] }
```

Use cheaper/wrong materials:

```
send CraftItem {
    recipe: 101,
    materials: [
        Material { item: Pebble, count: 1 }
    ]
}
```

Change the required number:

```
send CraftItem {
    recipe: 101,
    materials: [
        Material { item: DragonScale, count: 0 },
        Material { item: IronHilt, count: 1 },
    ]
}
```

Any variant causes the server to produce the Dragon Blade while consuming little
or nothing valuable.

## Naive failure

Crafting through the normal UI fails because the player lacks 3 Dragon Scales.

## Exploit recap

The craft accepted client-declared material counts, so the recipe completed
without spending real materials.
