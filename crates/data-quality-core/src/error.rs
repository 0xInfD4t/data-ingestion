use thiserror::Error;

/// Error type for the data-quality-core crate.
#[derive(Debug, Error)]
pub enum DqError {
    #[error("Contract parse error: {0}")]
    ContractParseError(String),

    #[error("Suite generation error in '{suite}': {reason}")]
    SuiteGenerationError { suite: String, reason: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("I/O error: {0}")]
    IoError(String),
}
