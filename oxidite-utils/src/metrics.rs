use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use once_cell::sync::Lazy;

/// Metrics tracked per route path
#[derive(Debug, Default)]
pub struct RouteMetrics {
    pub request_count: AtomicU64,
    pub success_count: AtomicU64,
    pub error_count: AtomicU64,
    pub total_duration_ms: AtomicU64,
}

/// Global registry for all monitored performance metrics
#[derive(Debug)]
pub struct MetricsRegistry {
    concurrent_requests: AtomicU64,
    route_metrics: RwLock<HashMap<String, Arc<RouteMetrics>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            concurrent_requests: AtomicU64::new(0),
            route_metrics: RwLock::new(HashMap::new()),
        }
    }

    pub fn increment_concurrent(&self) {
        self.concurrent_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_concurrent(&self) {
        self.concurrent_requests.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn concurrent_requests(&self) -> u64 {
        self.concurrent_requests.load(Ordering::Relaxed)
    }

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

    /// Returns a snapshot of all route metrics: (request_count, success_count, error_count, total_duration_ms)
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

pub static GLOBAL_METRICS: Lazy<MetricsRegistry> = Lazy::new(MetricsRegistry::new);
