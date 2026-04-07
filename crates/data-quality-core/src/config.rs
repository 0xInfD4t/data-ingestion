use serde::{Deserialize, Serialize};

/// Runtime configuration for the DQ generation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DqConfig {
    /// GX version string embedded in suite metadata. Default: "1.11.3"
    pub gx_version: String,
    /// Include the 17 baseline suites (1328 tests). Default: true
    pub include_baseline: bool,
    /// Include contract-specific suites. Default: true
    pub include_contract_specific: bool,
    /// If Some, only generate these named suites; None = all suites
    pub enabled_suites: Option<Vec<String>>,
    /// Suite names to explicitly skip
    pub disabled_suites: Vec<String>,
    /// Max acceptable null ratio for completeness expectations. Default: 0.05
    pub null_ratio_max: f64,
    /// Min acceptable uniqueness ratio. Default: 0.95
    pub uniqueness_min: f64,
    /// Z-score threshold for anomaly detection expectations. Default: 3.0
    pub z_score_max: f64,
    /// Historical mean for volume anomaly expectations (optional)
    pub historical_mean: Option<f64>,
    /// Historical std dev for volume anomaly expectations (optional)
    pub historical_std: Option<f64>,
}

impl Default for DqConfig {
    fn default() -> Self {
        Self {
            gx_version: "1.11.3".to_string(),
            include_baseline: true,
            include_contract_specific: true,
            enabled_suites: None,
            disabled_suites: vec![],
            null_ratio_max: 0.05,
            uniqueness_min: 0.95,
            z_score_max: 3.0,
            historical_mean: None,
            historical_std: None,
        }
    }
}
