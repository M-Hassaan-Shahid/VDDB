use metrics::{counter, gauge, histogram};
use std::time::Instant;
use metrics_exporter_prometheus::PrometheusBuilder;

pub struct QueryMetrics {
    start_time: Instant,
}

impl QueryMetrics {
    pub fn new() -> Self {
        QueryMetrics {
            start_time: Instant::now(),
        }
    }

    pub fn record_query_execution(&self, query_type: &str, success: bool) {
        let duration = self.start_time.elapsed();
        histogram!("query.execution_time", duration.as_secs_f64(), "type" => query_type.to_string());
        counter!("query.total", 1, "type" => query_type.to_string(), "success" => success.to_string());
    }

    pub fn record_table_operation(&self, operation: &str, table: &str) {
        counter!("table.operations", 1, "operation" => operation.to_string(), "table" => table.to_string());
    }

    pub fn record_index_operation(&self, operation: &str, index: &str) {
        counter!("index.operations", 1, "operation" => operation.to_string(), "index" => index.to_string());
    }

    pub fn record_memory_usage(&self, bytes: u64) {
        gauge!("memory.usage", bytes as f64);
    }

    pub fn record_cache_hits(&self, hits: u64, misses: u64) {
        counter!("cache.hits", hits);
        counter!("cache.misses", misses);
    }
}

pub fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(([127, 0, 0, 1], 9000))
        .install()?;
    Ok(())
} 