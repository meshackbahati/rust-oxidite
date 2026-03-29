# Socket.IO Bridge Adapter Guide

This guide shows how to keep existing Socket.IO clients while migrating backend APIs to Oxidite.

## Target scenario

- Existing frontend depends on Socket.IO event names and room semantics.
- You want Oxidite to own business APIs without breaking realtime clients.

## Recommended architecture

1. Keep current Socket.IO edge process (Node) temporarily.
2. Move domain logic/API routes to Oxidite.
3. Publish realtime domain events from Oxidite to Redis/Kafka.
4. Socket.IO edge consumes those events and emits unchanged client events.

## Event contract freeze

Before migration, freeze:

- room naming (`user:{id}`, `ctf:{eventId}`, `team:{id}`)
- event names (`leaderboard:update`, `notification:new`, etc.)
- payload shape and nullable fields

## Oxidite producer pattern

Use `oxidite-realtime` + queue/pubsub layer to emit canonical domain events.

```rust,ignore
use oxidite_realtime::{Event, EventType};

let event = Event::new(
    EventType::Custom("leaderboard:update".into()),
    serde_json::json!({"eventId": 42, "delta": 15})
);
```

## Bridge consumer pattern

In bridge service:

1. Consume Oxidite domain events.
2. Map to legacy Socket.IO event names.
3. Emit to existing rooms.
4. Log unmapped events as warnings.

## Backward compatibility checks

- Client contract tests for room/event/payload compatibility
- Replay test stream against staging clients
- Drop-rate and lag metrics on bridge consumer

## Cutover plan

1. Shadow mode: Oxidite emits but clients still served from legacy path.
2. Dual emit: compare payloads from both paths.
3. Flip write source to Oxidite.
4. Remove legacy emitters after stable release window.
