use thiserror::Error;

/// Top-level error type for the ingestion pipeline.
/// Wraps TransformError and OutputError for unified handling.
#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Parse error in {source_format} at '{field_path}': {message}")]
    ParseError {
        source_format: String,
        field_path: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Format detection failed: {reason}")]
    DetectionFailed { reason: String },

    #[error("Unresolved reference: '{reference}'")]
    UnresolvedReference { reference: String },

    #[error("Circular reference detected at path: {path}")]
    CircularReference { path: String },

    /// Only available on non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Transform failed: {0}")]
    Transform(#[from] TransformError),

    #[error("Output failed: {0}")]
    Output(#[from] OutputError),
}

impl IngestionError {
    /// Returns a stable, uppercase error code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedFormat { .. }   => "UNSUPPORTED_FORMAT",
            Self::ParseError { .. }          => "PARSE_ERROR",
            Self::DetectionFailed { .. }     => "DETECTION_FAILED",
            Self::UnresolvedReference { .. } => "UNRESOLVED_REFERENCE",
            Self::CircularReference { .. }   => "CIRCULAR_REFERENCE",
            #[cfg(not(target_arch = "wasm32"))]
            Self::IoError(_)                 => "IO_ERROR",
            Self::Transform(e)               => e.code(),
            Self::Output(e)                  => e.code(),
        }
    }

    /// Convenience constructor for parse errors.
    pub fn parse(
        source_format: impl Into<String>,
        field_path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            source_format: source_format.into(),
            field_path: field_path.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Convenience constructor for parse errors with a source error.
    pub fn parse_with_source(
        source_format: impl Into<String>,
        field_path: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::ParseError {
            source_format: source_format.into(),
            field_path: field_path.into(),
            message: message.into(),
            source: Some(source),
        }
    }
}

/// Errors from the IR → DataContract transformation stage.
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Invalid IR: {reason}")]
    InvalidIr { reason: String },

    #[error("Type resolution failed for field '{field_name}': {reason}")]
    TypeResolutionFailed { field_name: String, reason: String },

    #[error("Constraint extraction failed: {reason}")]
    ConstraintExtractionFailed { reason: String },
}

impl TransformError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidIr { .. }                  => "INVALID_IR",
            Self::TypeResolutionFailed { .. }       => "TYPE_RESOLUTION_FAILED",
            Self::ConstraintExtractionFailed { .. } => "CONSTRAINT_EXTRACTION_FAILED",
        }
    }
}

/// Errors from the DataContract → output serialization stage.
#[derive(Debug, Error)]
pub enum OutputError {
    #[error("Serialization to '{format}' failed: {reason}")]
    SerializationFailed { format: String, reason: String },

    #[error("Unsupported output format: '{format}'. Expected one of: json, yaml, xml, csv")]
    UnsupportedOutputFormat { format: String },

    #[error("Contract deserialization failed: {reason}")]
    DeserializationFailed { reason: String },
}

impl OutputError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::SerializationFailed { .. }    => "SERIALIZATION_FAILED",
            Self::UnsupportedOutputFormat { .. }=> "UNSUPPORTED_OUTPUT_FORMAT",
            Self::DeserializationFailed { .. }  => "DESERIALIZATION_FAILED",
        }
    }
}
