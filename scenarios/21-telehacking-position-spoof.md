# 21 — Blink Without Moving: Telehacking Position Spoof

Category: Movement
Bug class: Server trusts client position on interaction
Difficulty: ★★★

## Objective

Activate the shrine on the far island without crossing the bridge.

## Player-facing setup

The shrine is at x=100 across a broken bridge. Normal movement cannot reach it.
The packet list exposes both `Move` and `Interact` packets; the `Interact`
packet includes the client's current coordinates for animation prediction.

## Packet schemas

```
C->S Move     { x: Int, y: Int }
C->S Interact { target: Int, client_x: Int, client_y: Int }
S->C Position { x: Int, y: Int }
S->C ShrineState { id: Int, active: Bool }
```

## Initial state

```
player { x: 0, y: 0 }
shrine { id: 91, x: 100, y: 0, active: false }
bridge { x: 40..60, broken: true }
```

## Server rule / hidden bug

Movement normally tracks x=0 on the server, but the interaction range check uses
`client_x/client_y` from the `Interact` packet instead of the stored player
position. This allows "telehacking": spoofing a temporary remote position only
for the interaction.

## Intended exploit

```
send Interact { target: 91, client_x: 7, client_y: 3 }
```

The server believes the player is standing beside the shrine for the range check
and activates it, even though the authoritative position remains x=0.

## Naive failure

```
for x in 1..101 {
    send Move { x: x, y: 0 }
    sleep 20
}
```

Movement is blocked by the broken bridge.

## Exploit recap

The interaction packet carried its own position, letting you activate the shrine
without actually crossing.
