# Observability

Production services need structured logs, traces, and metrics.

## Logging

Use request-scoped IDs and structured JSON logs for correlation.

Recommended fields:

- request id
- route pattern
- HTTP method/status
- duration
- user id (if authenticated)

## Tracing

Instrument critical paths:

- DB queries and transactions
- outbound HTTP calls
- queue enqueue/dequeue
- websocket room operations

Use span boundaries around handlers and service-layer operations.

## Metrics

Track at minimum:

- request rate
- latency percentiles
- error rate by route
- queue depth and retry count
- cache hit/miss ratio

## Practical Deployment Notes

- Keep high-cardinality labels out of metric keys.
- Sample traces in high-throughput environments.
- Tie request IDs across logs and traces.
