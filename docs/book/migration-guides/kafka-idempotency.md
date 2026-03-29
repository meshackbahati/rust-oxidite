# Kafka Integration with Idempotent Consumers

Use this guide when migrating event workers from Node to Oxidite.

## Design rules

- Process messages at-least-once.
- Make handlers idempotent.
- Commit offsets only after durable side effects.

## Idempotency techniques

- Dedup table keyed by `event_id` or producer idempotency key.
- Transactional write pattern: business change + dedup marker together.
- Ignore duplicates as successful no-op.

## Recommended worker flow

1. Receive message.
2. Validate schema/version.
3. Begin DB transaction.
4. Check dedup marker.
5. Apply side effects if first-seen.
6. Persist dedup marker.
7. Commit transaction.
8. Commit Kafka offset.

## Failure handling

- Retry transient DB/network errors.
- Dead-letter poison messages with context.
- Expose lag/retry/dead-letter metrics.
