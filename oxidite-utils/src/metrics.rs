//! Request-level metrics registry
//!
//! Tracks request counts, success/error rates, and total duration per route.
//! Uses lock-free atomics for hot-path counters and a `RwLock` for route registration.
//!
//! # Examples
//!
//! ```rust
//! use oxidite_utils::metrics::MetricsRegistry;
//!
//! let registry = MetricsRegistry::new();
//! registry.record_request("/health", 5, true);
//! let snapshot = registry.get_snapshot();
//! assert_eq!(snapshot.get("/health").unwrap(), (1, 1, 0, 5));
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use once_cell::sync::Lazy;

/// Metrics tracked per route path
///
/// Fields use `AtomicU64` so recording metrics never blocks concurrent requests.
///
/// ```rust
/// use oxidite_utils::metrics::RouteMetrics;
///
/// let m = RouteMetrics::default();
/// m.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
/// assert_eq!(m.request_count.load(std::sync::atomic::Ordering::Relaxed), 1);
/// ```
#[derive(Debug, Default)]
pub struct RouteMetrics {
    pub request_count: AtomicU64,
    pub success_count: AtomicU64,
    pub error_count: AtomicU64,
    pub total_duration_ms: AtomicU64,
}

/// Global registry for all monitored performance metrics
///
/// Provides concurrent access to per-route counters and a global
/// concurrent-request gauge.
///
/// ```rust
/// use oxidite_utils::metrics::MetricsRegistry;
///
/// let registry = MetricsRegistry::new();
/// registry.increment_concurrent();
/// registry.record_request("/api/data", 42, true);
/// registry.decrement_concurrent();
/// assert_eq!(registry.concurrent_requests(), 0);
/// ```
#[derive(Debug)]
pub struct MetricsRegistry {
    concurrent_requests: AtomicU64,
    route_metrics: RwLock<HashMap<String, Arc<RouteMetrics>>>,
}

impl MetricsRegistry {
    /// Create an empty `MetricsRegistry` with no route metrics registered
    pub fn new() -> Self {
        Self {
            concurrent_requests: AtomicU64::new(0),
            route_metrics: RwLock::new(HashMap::new()),
        }
    }

    /// Atomically increment the concurrent request counter
    pub fn increment_concurrent(&self) {
        self.concurrent_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Atomically decrement the concurrent request counter
    pub fn decrement_concurrent(&self) {
        self.concurrent_requests.fetch_sub(1, Ordering::Relaxed);
    }

    /// Read the current concurrent request count
    pub fn concurrent_requests(&self) -> u64 {
        self.concurrent_requests.load(Ordering::Relaxed)
    }

    /// Record a single request's duration and success/failure status for a route path
    ///
    /// Uses a double-checked lock pattern: tries a read-lock first, then upgrades
    /// to a write-lock only if the route does not yet exist.
    ///
    /// ```rust
    /// use oxidite_utils::metrics::MetricsRegistry;
    ///
    /// let registry = MetricsRegistry::new();
    /// registry.record_request("/api/health", 10, true);
    /// registry.record_request("/api/health", 20, false);
    /// let snapshot = registry.get_snapshot();
    /// let (count, success, errors, _) = snapshot.get("/api/health").unwrap();
    /// assert_eq!(*count, 2);
    /// assert_eq!(*success, 1);
    /// assert_eq!(*errors, 1);
    /// ```
    pub fn record_request(&self, path: &str, duration_ms: u64, is_success: bool) {
        let route_metric: Arc<RouteMetrics> = {
            let read = self.route_metrics.read();
            if let Some(metric) = read.get(path) {
                metric.clone()
            } else {
                drop(read);
                let mut write = self.route_metrics.write();
                write.entry(path.to_string())
                    .or_insert_with(|| Arc::new(RouteMetrics::default()))
                    .clone()
            }
        };

        route_metric.request_count.fetch_add(1, Ordering::Relaxed);
        route_metric.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        if is_success {
            route_metric.success_count.fetch_add(1, Ordering::Relaxed);
        } else {
            route_metric.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Return a snapshot of all route metrics
    ///
    /// Returns `(request_count, success_count, error_count, total_duration_ms)` per path.
    ///
    /// ```rust
    /// use oxidite_utils::metrics::MetricsRegistry;
    ///
    /// let registry = MetricsRegistry::new();
    /// registry.record_request("/a", 5, true);
    /// let snapshot = registry.get_snapshot();
    /// assert_eq!(snapshot.len(), 1);
    /// ```
    pub fn get_snapshot(&self) -> HashMap<String, (u64, u64, u64, u64)> {
        let read = self.route_metrics.read();
        read.iter()
            .map(|(path, metric): (&String, &Arc<RouteMetrics>)| {
                (
                    path.clone(),
                    (
                        metric.request_count.load(Ordering::Relaxed),
                        metric.success_count.load(Ordering::Relaxed),
                        metric.error_count.load(Ordering::Relaxed),
                        metric.total_duration_ms.load(Ordering::Relaxed),
                    ),
                )
            })
            .collect()
    }
}

/// Global singleton metrics registry for application-wide use
///
/// ```rust
/// use oxidite_utils::metrics::GLOBAL_METRICS;
///
/// GLOBAL_METRICS.record_request("/health", 3, true);
/// ```
pub static GLOBAL_METRICS: Lazy<MetricsRegistry> = Lazy::new(MetricsRegistry::new);
