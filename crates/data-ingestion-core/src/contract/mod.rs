//! Data contract transformation engine.
//!
//! Converts a normalized [`IrDocument`](crate::ir::model::IrDocument) into a
//! [`DataContract`] that can be serialized to JSON, YAML, XML, or CSV.
//!
//! ## Pipeline
//!
//! ```text
//! IrDocument → ContractBuilder → DataContract → ContractValidator
//! ```

pub mod builder;
pub mod enricher;
pub mod model;
pub mod validator;

// ── Public re-exports ─────────────────────────────────────────────────────────

pub use builder::{ContractBuilder, ContractBuilderConfig};
pub use enricher::MetadataEnricher;
pub use model::{
    ContractField, DataClassification, DataContract, FieldConstraint, ForeignKeyRef, LineageInfo,
    LogicalType, QualityInfo, SlaInfo,
};
pub use validator::{ContractValidator, ValidationResult};
