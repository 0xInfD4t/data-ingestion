//! # data-ingestion-core
//!
//! Pure-Rust library for ingesting heterogeneous data formats and producing
//! a unified Intermediate Representation (IR), then transforming it into a
//! typed [`DataContract`] that can be serialized to JSON, YAML, XML, or CSV.
//!
//! ## Pipeline
//!
//! ```text
//! Raw bytes → FormatDetector → FormatReader → IrDocument → IrNormalizer → IrDocument
//!           → ContractBuilder → DataContract → ContractValidator
//!           → serialize (JSON | YAML | XML | CSV)
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use data_ingestion_core::{ingest, ingestion::FormatHint};
//!
//! let bytes = b"[{\"id\": 1, \"name\": \"Alice\"}]";
//! let hint = FormatHint::from_filename("data.json");
//! let ir_doc = ingest(bytes, &hint).unwrap();
//! println!("Source format: {:?}", ir_doc.source_format);
//! ```

pub mod contract;
pub mod error;
pub mod ingestion;
pub mod ir;
pub mod output;

// ── Public re-exports ─────────────────────────────────────────────────────────

pub use error::{IngestionError, OutputError, TransformError};

pub use ir::model::{
    IrArray, IrConstraint, IrDocument, IrEnum, IrField, IrNode, IrObject, IrType, SourceFormat,
};
pub use ir::normalizer::IrNormalizer;

pub use ingestion::detector::FormatDetector;
pub use ingestion::traits::{FormatHint, FormatReader};

// Contract module re-exports
pub use contract::model::{
    ContractField, DataClassification, DataContract, FieldConstraint, LineageInfo, LogicalType,
    QualityInfo, SlaInfo,
};
pub use contract::builder::{ContractBuilder, ContractBuilderConfig};
pub use contract::validator::{ContractValidator, ValidationResult};

// Output module re-exports
pub use output::{serialize as serialize_contract, OutputFormat};

// ── Main entry point ──────────────────────────────────────────────────────────

/// Ingest raw bytes and return a normalized `IrDocument`.
///
/// This is the primary entry point for the ingestion pipeline.
/// It performs format detection, reading, and normalization in one call.
///
/// # Arguments
/// * `content` - Raw file bytes (no filesystem access; caller provides bytes)
/// * `hint`    - Format hint (filename, MIME type, or explicit format override)
///
/// # Errors
/// Returns [`IngestionError`] if format detection, parsing, or normalization fails.
///
/// # Example
/// ```rust,no_run
/// use data_ingestion_core::{ingest, ingestion::FormatHint};
///
/// let bytes = b"[{\"id\": 1, \"name\": \"Alice\"}]";
/// let hint = FormatHint::from_filename("sample.json");
/// let doc = ingest(bytes, &hint).unwrap();
/// ```
pub fn ingest(content: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
    log::debug!(
        "ingest: {} bytes, hint filename={:?}",
        content.len(),
        hint.filename
    );

    // Step 1: Detect format and read into raw IrDocument
    let raw_doc = FormatDetector::detect_and_read(content, hint)?;

    log::debug!(
        "ingest: read complete, source_format={:?}, normalizing...",
        raw_doc.source_format
    );

    // Step 2: Normalize the IrDocument
    let normalized = IrNormalizer::normalize(raw_doc)?;

    log::debug!("ingest: normalization complete");

    Ok(normalized)
}

/// Detect the format of raw bytes without full parsing.
///
/// Useful for format identification without the overhead of full ingestion.
pub fn detect_format(content: &[u8], hint: &FormatHint) -> SourceFormat {
    FormatDetector::detect(content, hint)
}

/// Full pipeline: ingest bytes → parse IR → build [`DataContract`].
///
/// Combines [`ingest`] and [`ContractBuilder::build`] into a single call.
///
/// # Arguments
/// * `content`      - Raw file bytes
/// * `format_hint`  - Optional filename or format string hint
/// * `source_path`  - Optional source path (used for lineage metadata)
/// * `config`       - [`ContractBuilderConfig`] controlling transformation behaviour
///
/// # Errors
/// Returns a boxed error if any stage of the pipeline fails.
pub fn process(
    content: &[u8],
    format_hint: Option<&str>,
    source_path: Option<&str>,
    config: ContractBuilderConfig,
) -> Result<DataContract, Box<dyn std::error::Error>> {
    log::debug!(
        "process: {} bytes, format_hint={:?}, source_path={:?}",
        content.len(),
        format_hint,
        source_path
    );

    let hint = FormatHint {
        filename: format_hint
            .or(source_path)
            .map(|s| {
                // Extract just the filename portion if a full path was given
                std::path::Path::new(s)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(s)
                    .to_string()
            }),
        ..Default::default()
    };

    let ir_doc = ingest(content, &hint)?;
    let builder = ContractBuilder::new(config);
    let contract = builder.build(&ir_doc)?;

    log::debug!(
        "process: produced contract '{}' with {} fields",
        contract.name,
        contract.fields.len()
    );

    Ok(contract)
}

/// Serialize a [`DataContract`] to the specified output format.
///
/// # Errors
/// Returns [`OutputError`] if serialization fails.
pub fn to_format(
    contract: &DataContract,
    format: OutputFormat,
) -> Result<Vec<u8>, OutputError> {
    output::serialize(contract, format)
}

// ── File I/O helpers (native only) ────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
/// Ingest a file from disk by path.
///
/// Only available on non-WASM targets. Reads the file and delegates to [`ingest`].
pub fn ingest_file(path: &std::path::Path) -> Result<IrDocument, IngestionError> {
    let content = std::fs::read(path)?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from);
    let hint = FormatHint {
        filename,
        ..Default::default()
    };
    ingest(&content, &hint)
}

#[cfg(not(target_arch = "wasm32"))]
/// Process a file from disk into a [`DataContract`].
///
/// Only available on non-WASM targets. Reads the file and delegates to [`process`].
pub fn process_file(
    path: &std::path::Path,
    config: ContractBuilderConfig,
) -> Result<DataContract, Box<dyn std::error::Error>> {
    let content = std::fs::read(path)?;
    let path_str = path.to_str().unwrap_or("");
    process(&content, None, Some(path_str), config)
}
