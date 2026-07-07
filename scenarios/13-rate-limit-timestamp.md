# 13 — Same Millisecond: Timestamp Rate-Limit Bypass

Category: Rate limiting
Bug class: Dedupe by timestamp only
Difficulty: ★★☆

## Objective

Overload the crystal.

## Player-facing setup

The crystal has 10 HP. Each `Zap` deals 1 damage. Normal clients can zap once
every 50ms, but the crystal fully regenerates every 200ms.

## Packet schemas

```
C->S Zap { target: Int, client_time_ms: Int }
S->C Damage { target: Int, amount: Int }
S->C Regen { target: Int, hp: Int }
S->C Death { id: Int }
```

## Initial state

```
player  { id: 0 }
crystal { id: 1, hp: 10, regen_every_ms: 200 }
```

## Server rule / hidden bug

The server's rate limiter stores the last accepted client timestamp, but allows
multiple packets with the exact same timestamp because it only rejects strictly
increasing deltas below 50ms.

## Intended exploit

```
send_batch {
    for i in 1..11 {
        Zap { target: 1, client_time_ms: 0 }
    }
}
```

All zaps share the same timestamp and pass the flawed limiter in one frame.

## Naive failure

```
for i in 1..11 {
    send Zap { target: 1, client_time_ms: now() }
    sleep 50
}
```

The crystal regenerates before enough damage accumulates.

## Exploit recap

Repeated packets shared the same client timestamp bucket, bypassing the
limiter's distinct-time assumption.
