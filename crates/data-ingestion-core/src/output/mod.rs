//! Multi-format output serializers for [`DataContract`](crate::contract::model::DataContract).
//!
//! Supported formats: JSON, YAML, XML, CSV.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use data_ingestion_core::output::{OutputFormat, serialize};
//! # use data_ingestion_core::contract::model::DataContract;
//! # use std::collections::HashMap;
//! # let contract = DataContract {
//! #     id: "id".to_string(), name: "n".to_string(), version: "1.0.0".to_string(),
//! #     description: None, owner: None, domain: None, source_format: "json".to_string(),
//! #     fields: vec![], metadata: HashMap::new(), sla: None, lineage: None, quality: None,
//! #     created_at: None, tags: vec![],
//! # };
//! let bytes = serialize(&contract, OutputFormat::Json).unwrap();
//! ```

pub mod csv;
pub mod json;
pub mod xml;
pub mod yaml;

use crate::contract::model::DataContract;
use crate::error::OutputError;

// ── OutputFormat ──────────────────────────────────────────────────────────────

/// Supported output serialization formats.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Yaml,
    Xml,
    Csv,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Xml  => write!(f, "xml"),
            Self::Csv  => write!(f, "csv"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = OutputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            "xml"  => Ok(Self::Xml),
            "csv"  => Ok(Self::Csv),
            other  => Err(OutputError::UnsupportedOutputFormat {
                format: other.to_string(),
            }),
        }
    }
}

// ── Dispatch function ─────────────────────────────────────────────────────────

/// Serialize a [`DataContract`] to the specified format, returning UTF-8 bytes.
///
/// # Errors
/// Returns [`OutputError`] if serialization fails.
pub fn serialize(contract: &DataContract, format: OutputFormat) -> Result<Vec<u8>, OutputError> {
    log::debug!("output::serialize: format={}", format);
    match format {
        OutputFormat::Json => json::to_json(contract),
        OutputFormat::Yaml => yaml::to_yaml(contract),
        OutputFormat::Xml  => xml::to_xml(contract),
        OutputFormat::Csv  => csv::to_csv(contract),
    }
}
