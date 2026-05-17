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

Oxidite includes a high-performance, atomic-backed telemetry system for tracking application performance metrics. It captures request counters, active concurrent requests, latencies, and success/error status across all endpoints.

### Global Metrics Registry

The `GLOBAL_METRICS` registry (`oxidite::utils::GLOBAL_METRICS`) is a concurrent, lock-free global registry that records real-time performance statistics:

```rust
use oxidite::utils::GLOBAL_METRICS;

// Get the current number of concurrent active HTTP requests
let active_requests = GLOBAL_METRICS.concurrent_requests();
println!("Current concurrent active requests: {}", active_requests);

// Get a snapshot of all route statistics
// Returns: HashMap<String, (request_count, success_count, error_count, total_duration_ms)>
let metrics_snapshot = GLOBAL_METRICS.get_snapshot();
for (path, (reqs, successes, errors, duration)) in metrics_snapshot {
    let avg_latency = if reqs > 0 { duration / reqs } else { 0 };
    println!("Route [{}]: {} reqs, {} success, {} error, {}ms avg", path, reqs, successes, errors, avg_latency);
}
```

### Metrics Middleware Layer

To automatically track metrics for all incoming requests, register the `MetricsLayer` middleware from `oxidite::middleware` in your router definition:

```rust
use oxidite::core::Router;
use oxidite::middleware::MetricsLayer;

let app = Router::new()
    // Register the advanced telemetry metrics middleware
    .layer(MetricsLayer)
    .route("/users", get(list_users));
```

## Practical Deployment Notes

- **Metrics Performance:** The metrics registry is designed using lock-free atomic registers (`AtomicU64`) and fine-grained concurrent maps, keeping processing latency below 1 microsecond.
- **Keep high-cardinality labels out of metric keys:** Map dynamic path parameters to templates (e.g. `/users/:id` rather than raw `/users/123`) to avoid registry key explosions.
- **Sample traces in high-throughput environments.**
- **Tie request IDs across logs and traces.**

