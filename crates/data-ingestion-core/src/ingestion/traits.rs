use crate::error::IngestionError;
use crate::ir::model::{IrDocument, SourceFormat};

/// Format hint provided to readers and the detector.
#[derive(Debug, Default, Clone)]
pub struct FormatHint {
    /// Original filename — used for extension-based detection.
    pub filename: Option<String>,
    /// MIME type if known (e.g. "application/json").
    pub mime_type: Option<String>,
    /// Explicit format override — skips auto-detection entirely.
    pub explicit_format: Option<SourceFormat>,
}

impl FormatHint {
    /// Create a hint from a filename.
    pub fn from_filename(filename: impl Into<String>) -> Self {
        Self {
            filename: Some(filename.into()),
            ..Default::default()
        }
    }

    /// Create a hint with an explicit format override.
    pub fn from_format(format: SourceFormat) -> Self {
        Self {
            explicit_format: Some(format),
            ..Default::default()
        }
    }

    /// Create a hint from a filename and explicit format.
    pub fn new(filename: Option<String>, mime_type: Option<String>, explicit_format: Option<SourceFormat>) -> Self {
        Self { filename, mime_type, explicit_format }
    }

    /// Extract the file extension from the filename, if present.
    pub fn extension(&self) -> Option<&str> {
        self.filename.as_deref().and_then(|f| {
            let path = std::path::Path::new(f);
            path.extension().and_then(|e| e.to_str())
        })
    }
}

/// All format readers implement this trait.
pub trait FormatReader: Send + Sync {
    /// Returns true if this reader can handle the given hint.
    fn can_read(&self, hint: &FormatHint) -> bool;

    /// Parse input bytes into an IrDocument.
    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError>;
}
