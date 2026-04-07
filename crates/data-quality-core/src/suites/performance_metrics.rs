use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct PerformanceMetricsSuite;

impl SuiteGenerator for PerformanceMetricsSuite {
    fn suite_name(&self) -> &str { "performance_metrics_suite" }
    fn category(&self) -> &str { "performance" }
    fn test_id_prefix(&self) -> &str { "PM" }
    fn test_id_start(&self) -> usize { 1116 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // PM1116-PM1145: Performance metric range checks (30 tests)
        let perf_ranges: &[(&str, f64, f64)] = &[
            ("query_execution_time_ms", 0.0, 30_000.0),
            ("api_response_time_ms", 0.0, 10_000.0),
            ("data_load_time_seconds", 0.0, 3600.0),
            ("batch_processing_time_seconds", 0.0, 86400.0),
            ("etl_duration_minutes", 0.0, 1440.0),
            ("ingestion_rate_records_per_second", 0.0, 1_000_000.0),
            ("throughput_mb_per_second", 0.0, 10_000.0),
            ("cpu_utilization_percent", 0.0, 100.0),
            ("memory_utilization_percent", 0.0, 100.0),
            ("disk_io_mb_per_second", 0.0, 10_000.0),
            ("network_latency_ms", 0.0, 5_000.0),
            ("cache_hit_rate", 0.0, 1.0),
            ("error_rate", 0.0, 1.0),
            ("retry_rate", 0.0, 1.0),
            ("timeout_rate", 0.0, 1.0),
            ("success_rate", 0.0, 1.0),
            ("availability_percent", 0.0, 100.0),
            ("uptime_percent", 0.0, 100.0),
            ("sla_compliance_rate", 0.0, 1.0),
            ("data_freshness_hours", 0.0, 168.0),
            ("replication_lag_seconds", 0.0, 3600.0),
            ("queue_depth", 0.0, 1_000_000.0),
            ("concurrent_connections", 0.0, 10_000.0),
            ("active_sessions", 0.0, 100_000.0),
            ("pending_jobs", 0.0, 100_000.0),
            ("failed_jobs_count", 0.0, 10_000.0),
            ("retry_count", 0.0, 100.0),
            ("dead_letter_count", 0.0, 10_000.0),
            ("checkpoint_lag_seconds", 0.0, 3600.0),
            ("compaction_time_seconds", 0.0, 86400.0),
        ];
        for (i, (col, min_v, max_v)) in perf_ranges.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1116 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // PM1146-PM1175: Performance metric not-null and type checks (30 tests)
        let perf_fields = [
            "job_id", "job_name", "job_type", "start_time", "end_time",
            "status", "records_processed", "records_failed", "records_skipped", "duration_seconds",
            "source_system", "target_system", "pipeline_name", "pipeline_version", "run_id",
            "batch_id", "partition_id", "shard_id", "worker_id", "node_id",
            "environment", "region", "availability_zone", "cluster_id", "service_name",
            "version", "build_number", "deployment_id", "release_tag", "commit_hash",
        ];
        for (i, col) in perf_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_be_null".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1146 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 60, "PerformanceMetricsSuite must produce 60 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_performance_metrics_suite_count() {
        let suite = PerformanceMetricsSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 60, "PM1116-PM1175 must produce 60 tests");
    }
}
