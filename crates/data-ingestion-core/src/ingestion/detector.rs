use crate::error::IngestionError;
use crate::ingestion::traits::FormatHint;
use crate::ir::model::SourceFormat;

/// Three-pass format detector.
///
/// Pass 1: Explicit format hint override
/// Pass 2: File extension from filename
/// Pass 3: Content sniffing (first 512 bytes)
pub struct FormatDetector;

impl FormatDetector {
    /// Detect the source format from input bytes and a hint.
    pub fn detect(input: &[u8], hint: &FormatHint) -> SourceFormat {
        // Pass 1: Explicit override
        if let Some(explicit) = &hint.explicit_format {
            log::debug!("Format detection: explicit override → {:?}", explicit);
            return explicit.clone();
        }

        // Pass 1b: MIME type hint
        if let Some(mime) = &hint.mime_type {
            if let Some(fmt) = detect_from_mime(mime) {
                log::debug!("Format detection: MIME type '{}' → {:?}", mime, fmt);
                return fmt;
            }
        }

        // Pass 2: File extension
        if let Some(ext) = hint.extension() {
            if let Some(fmt) = detect_from_extension(ext) {
                log::debug!("Format detection: extension '.{}' → {:?}", ext, fmt);
                return fmt;
            }
        }

        // Pass 3: Content sniffing
        let sniff_bytes = &input[..input.len().min(512)];
        let fmt = sniff_content(sniff_bytes);
        log::debug!("Format detection: content sniff → {:?}", fmt);
        fmt
    }

    /// Detect format and return the appropriate reader.
    pub fn detect_and_read(
        input: &[u8],
        hint: &FormatHint,
    ) -> Result<crate::ir::model::IrDocument, IngestionError> {
        use crate::ingestion::{
            reader_csv::CsvReader,
            reader_json::JsonDatasetReader,
            reader_json_schema::JsonSchemaReader,
            reader_xml::XmlReader,
            reader_xsd::XsdReader,
            reader_yaml::YamlReader,
            traits::FormatReader,
        };

        let format = Self::detect(input, hint);

        let reader: Box<dyn FormatReader> = match &format {
            SourceFormat::JsonSchema    => Box::new(JsonSchemaReader),
            SourceFormat::JsonDataset   => Box::new(JsonDatasetReader),
            SourceFormat::Xsd           => Box::new(XsdReader),
            SourceFormat::Xml           => Box::new(XmlReader),
            SourceFormat::DataDictionary=> Box::new(CsvReader),
            SourceFormat::DataStructure => Box::new(YamlReader),
            SourceFormat::DataSchema    => Box::new(YamlReader),
            SourceFormat::Unknown       => {
                return Err(IngestionError::DetectionFailed {
                    reason: "Could not determine format from content or hint. \
                             Provide a filename hint or explicit_format."
                        .to_string(),
                });
            }
        };

        reader.read(input, hint)
    }
}

// ── Pass 1b: MIME type detection ──────────────────────────────────────────────

fn detect_from_mime(mime: &str) -> Option<SourceFormat> {
    let mime_lower = mime.to_lowercase();
    match mime_lower.as_str() {
        "application/json" | "text/json"                => Some(SourceFormat::JsonDataset),
        "application/schema+json"                       => Some(SourceFormat::JsonSchema),
        "application/xml" | "text/xml"                  => Some(SourceFormat::Xml),
        "text/csv" | "application/csv"                  => Some(SourceFormat::DataDictionary),
        "application/yaml" | "text/yaml" | "text/x-yaml"=> Some(SourceFormat::DataStructure),
        _ => None,
    }
}

// ── Pass 2: Extension detection ───────────────────────────────────────────────

fn detect_from_extension(ext: &str) -> Option<SourceFormat> {
    match ext.to_lowercase().as_str() {
        "xsd"           => Some(SourceFormat::Xsd),
        "xml"           => Some(SourceFormat::Xml),
        "csv"           => Some(SourceFormat::DataDictionary),
        "yaml" | "yml"  => Some(SourceFormat::DataStructure),
        // .json is ambiguous — defer to content sniffing
        "json"          => None,
        _               => None,
    }
}

// ── Pass 3: Content sniffing ──────────────────────────────────────────────────

fn sniff_content(bytes: &[u8]) -> SourceFormat {
    let trimmed = trim_bom(bytes);
    let text = std::str::from_utf8(trimmed).unwrap_or("");
    let trimmed_text = text.trim_start();

    // XSD: starts with <?xml and contains xs:schema or xsd:schema
    if trimmed_text.starts_with("<?xml") || trimmed_text.starts_with('<') {
        let lower = trimmed_text.to_lowercase();
        if lower.contains("xs:schema")
            || lower.contains("xsd:schema")
            || lower.contains("xmlns:xs=")
            || lower.contains("xmlns:xsd=")
        {
            return SourceFormat::Xsd;
        }
        return SourceFormat::Xml;
    }

    // JSON: starts with { or [
    if trimmed_text.starts_with('{') || trimmed_text.starts_with('[') {
        // Check for JSON Schema indicator
        if trimmed_text.contains("\"$schema\"") || trimmed_text.contains("\"$defs\"") {
            return SourceFormat::JsonSchema;
        }
        return SourceFormat::JsonDataset;
    }

    // YAML: starts with --- or has YAML-like structure
    if trimmed_text.starts_with("---") {
        // Check if it looks like a data dictionary (list of field objects)
        if trimmed_text.contains("field_name:")
            || trimmed_text.contains("name:")
            || trimmed_text.contains("data_type:")
            || trimmed_text.contains("type:")
        {
            return SourceFormat::DataDictionary;
        }
        return SourceFormat::DataStructure;
    }

    // CSV: check for header row with field/name/type columns
    if looks_like_csv(trimmed_text) {
        return SourceFormat::DataDictionary;
    }

    // YAML without --- marker
    if trimmed_text.contains(':') && !trimmed_text.contains('<') {
        return SourceFormat::DataStructure;
    }

    SourceFormat::Unknown
}

/// Strip UTF-8 BOM if present.
fn trim_bom(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Heuristic: does the content look like a CSV file?
fn looks_like_csv(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or("");
    if first_line.is_empty() {
        return false;
    }

    // Must have at least one comma
    if !first_line.contains(',') {
        return false;
    }

    // Check for data dictionary column names
    let dict_columns = [
        "field_name", "column_name", "name", "field", "attribute",
        "data_type", "type", "dtype", "logical_type",
        "description", "desc", "comment",
        "nullable", "required", "is_nullable",
    ];

    let columns: Vec<&str> = first_line.split(',').collect();
    let matching = columns.iter().filter(|col| {
        let col_lower = col.trim().trim_matches('"').to_lowercase();
        dict_columns.iter().any(|&dc| col_lower == dc)
    }).count();

    // At least 2 matching column names → data dictionary
    matching >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_xsd_from_extension() {
        let hint = FormatHint::from_filename("schema.xsd");
        let fmt = FormatDetector::detect(b"", &hint);
        assert_eq!(fmt, SourceFormat::Xsd);
    }

    #[test]
    fn test_detect_xml_from_extension() {
        let hint = FormatHint::from_filename("data.xml");
        let fmt = FormatDetector::detect(b"", &hint);
        assert_eq!(fmt, SourceFormat::Xml);
    }

    #[test]
    fn test_detect_csv_from_extension() {
        let hint = FormatHint::from_filename("dict.csv");
        let fmt = FormatDetector::detect(b"", &hint);
        assert_eq!(fmt, SourceFormat::DataDictionary);
    }

    #[test]
    fn test_detect_json_schema_from_content() {
        let content = br#"{"$schema": "http://json-schema.org/draft-07/schema#", "type": "object"}"#;
        let hint = FormatHint::default();
        let fmt = FormatDetector::detect(content, &hint);
        assert_eq!(fmt, SourceFormat::JsonSchema);
    }

    #[test]
    fn test_detect_json_dataset_from_content() {
        let content = br#"[{"id": 1, "name": "Alice"}]"#;
        let hint = FormatHint::default();
        let fmt = FormatDetector::detect(content, &hint);
        assert_eq!(fmt, SourceFormat::JsonDataset);
    }

    #[test]
    fn test_detect_xsd_from_content() {
        let content = b"<?xml version=\"1.0\"?><xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"></xs:schema>";
        let hint = FormatHint::default();
        let fmt = FormatDetector::detect(content, &hint);
        assert_eq!(fmt, SourceFormat::Xsd);
    }

    #[test]
    fn test_detect_xml_from_content() {
        let content = b"<?xml version=\"1.0\"?><root><item>value</item></root>";
        let hint = FormatHint::default();
        let fmt = FormatDetector::detect(content, &hint);
        assert_eq!(fmt, SourceFormat::Xml);
    }

    #[test]
    fn test_explicit_override() {
        let hint = FormatHint::from_format(SourceFormat::Xsd);
        let fmt = FormatDetector::detect(b"[1,2,3]", &hint);
        assert_eq!(fmt, SourceFormat::Xsd);
    }
}
