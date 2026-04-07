use std::collections::{HashMap, HashSet};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::error::IngestionError;
use crate::ingestion::traits::{FormatHint, FormatReader};
use crate::ir::model::{
    IrConstraint, IrDocument, IrEnum, IrField, IrNode, IrObject, IrType, SourceFormat,
};

/// Reads XSD (XML Schema Definition) files.
pub struct XsdReader;

impl FormatReader for XsdReader {
    fn can_read(&self, hint: &FormatHint) -> bool {
        if let Some(ext) = hint.extension() {
            if ext.eq_ignore_ascii_case("xsd") {
                return true;
            }
        }
        if let Some(explicit) = &hint.explicit_format {
            return matches!(explicit, SourceFormat::Xsd);
        }
        true
    }

    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
        let mut parser = XsdParser::new();
        let root = parser.parse(input)?;
        Ok(IrDocument::new(
            SourceFormat::Xsd,
            hint.filename.clone(),
            root,
        ))
    }
}

// ── XSD Parser ────────────────────────────────────────────────────────────────

struct XsdParser {
    /// Named complex types collected in first pass
    complex_types: HashMap<String, IrObject>,
    /// Named simple types collected in first pass
    simple_types: HashMap<String, IrType>,
    /// Visited set for circular reference detection
    visited: HashSet<String>,
}

impl XsdParser {
    fn new() -> Self {
        Self {
            complex_types: HashMap::new(),
            simple_types: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    fn parse(&mut self, input: &[u8]) -> Result<IrNode, IngestionError> {
        // Two-pass parsing:
        // Pass 1: collect all named complexType and simpleType definitions
        self.collect_type_definitions(input)?;
        // Pass 2: build the root IrNode from top-level elements
        self.build_root(input)
    }

    // ── Pass 1: Collect type definitions ─────────────────────────────────────

    fn collect_type_definitions(&mut self, input: &[u8]) -> Result<(), IngestionError> {
        let mut reader = Reader::from_reader(input);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut depth = 0usize;
        let mut in_complex_type: Option<(String, usize)> = None;
        let mut in_simple_type: Option<(String, usize)> = None;
        let mut current_complex_obj: Option<IrObject> = None;
        let mut current_simple_base: Option<IrType> = None;
        let mut current_constraints: Vec<IrConstraint> = Vec::new();
        let mut current_enum_values: Vec<serde_json::Value> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e);
                    depth += 1;

                    match local.as_str() {
                        "complexType" => {
                            if let Some(name) = attr_value(e, "name") {
                                in_complex_type = Some((name.clone(), depth));
                                current_complex_obj = Some(IrObject::new(Some(name)));
                            }
                        }
                        "simpleType" => {
                            if let Some(name) = attr_value(e, "name") {
                                in_simple_type = Some((name, depth));
                                current_simple_base = None;
                                current_constraints.clear();
                                current_enum_values.clear();
                            }
                        }
                        "element" => {
                            if let Some((_, ct_depth)) = &in_complex_type {
                                if depth == ct_depth + 2 {
                                    // Direct child element of complexType/sequence/all
                                    if let Some(obj) = &mut current_complex_obj {
                                        let field = self.element_to_field(e);
                                        obj.fields.push(field);
                                    }
                                }
                            }
                        }
                        "attribute" => {
                            if let Some((_, ct_depth)) = &in_complex_type {
                                if depth > *ct_depth {
                                    if let Some(obj) = &mut current_complex_obj {
                                        let field = self.attribute_to_field(e);
                                        obj.fields.push(field);
                                    }
                                }
                            }
                        }
                        "restriction" => {
                            if in_simple_type.is_some() {
                                if let Some(base) = attr_value(e, "base") {
                                    current_simple_base = Some(xsd_type_to_ir(&base));
                                }
                            }
                        }
                        "enumeration" => {
                            if in_simple_type.is_some() {
                                if let Some(val) = attr_value(e, "value") {
                                    current_enum_values.push(serde_json::Value::String(val));
                                }
                            }
                        }
                        "minLength" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<u64>().ok()) {
                                    current_constraints.push(IrConstraint::MinLength(v));
                                }
                            }
                        }
                        "maxLength" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<u64>().ok()) {
                                    current_constraints.push(IrConstraint::MaxLength(v));
                                }
                            }
                        }
                        "pattern" => {
                            if in_simple_type.is_some() {
                                if let Some(p) = attr_value(e, "value") {
                                    current_constraints.push(IrConstraint::Pattern(p));
                                }
                            }
                        }
                        "minInclusive" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<f64>().ok()) {
                                    current_constraints.push(IrConstraint::Minimum(v));
                                }
                            }
                        }
                        "maxInclusive" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<f64>().ok()) {
                                    current_constraints.push(IrConstraint::Maximum(v));
                                }
                            }
                        }
                        "minExclusive" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<f64>().ok()) {
                                    current_constraints.push(IrConstraint::ExclusiveMinimum(v));
                                }
                            }
                        }
                        "maxExclusive" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<f64>().ok()) {
                                    current_constraints.push(IrConstraint::ExclusiveMaximum(v));
                                }
                            }
                        }
                        "totalDigits" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value") {
                                    current_constraints.push(IrConstraint::Custom(
                                        "totalDigits".to_string(),
                                        serde_json::Value::String(v),
                                    ));
                                }
                            }
                        }
                        "fractionDigits" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value") {
                                    current_constraints.push(IrConstraint::Custom(
                                        "fractionDigits".to_string(),
                                        serde_json::Value::String(v),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Ok(Event::Empty(ref e)) => {
                    let local = local_name(e);
                    match local.as_str() {
                        "element" => {
                            if let Some((_, ct_depth)) = &in_complex_type {
                                if depth + 1 == ct_depth + 2 || depth + 1 > *ct_depth {
                                    if let Some(obj) = &mut current_complex_obj {
                                        let field = self.element_to_field(e);
                                        obj.fields.push(field);
                                    }
                                }
                            }
                        }
                        "attribute" => {
                            if let Some((_, ct_depth)) = &in_complex_type {
                                if depth + 1 > *ct_depth {
                                    if let Some(obj) = &mut current_complex_obj {
                                        let field = self.attribute_to_field(e);
                                        obj.fields.push(field);
                                    }
                                }
                            }
                        }
                        "enumeration" => {
                            if in_simple_type.is_some() {
                                if let Some(val) = attr_value(e, "value") {
                                    current_enum_values.push(serde_json::Value::String(val));
                                }
                            }
                        }
                        "minLength" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<u64>().ok()) {
                                    current_constraints.push(IrConstraint::MinLength(v));
                                }
                            }
                        }
                        "maxLength" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<u64>().ok()) {
                                    current_constraints.push(IrConstraint::MaxLength(v));
                                }
                            }
                        }
                        "pattern" => {
                            if in_simple_type.is_some() {
                                if let Some(p) = attr_value(e, "value") {
                                    current_constraints.push(IrConstraint::Pattern(p));
                                }
                            }
                        }
                        "minInclusive" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<f64>().ok()) {
                                    current_constraints.push(IrConstraint::Minimum(v));
                                }
                            }
                        }
                        "maxInclusive" => {
                            if in_simple_type.is_some() {
                                if let Some(v) = attr_value(e, "value").and_then(|s| s.parse::<f64>().ok()) {
                                    current_constraints.push(IrConstraint::Maximum(v));
                                }
                            }
                        }
                        "restriction" => {
                            if in_simple_type.is_some() {
                                if let Some(base) = attr_value(e, "base") {
                                    current_simple_base = Some(xsd_type_to_ir(&base));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Ok(Event::End(ref e)) => {
                    let local = std::str::from_utf8(e.name().as_ref())
                        .unwrap_or("")
                        .split(':')
                        .last()
                        .unwrap_or("")
                        .to_string();

                    if local == "complexType" {
                        if let Some((name, ct_depth)) = in_complex_type.take() {
                            if depth == ct_depth {
                                if let Some(obj) = current_complex_obj.take() {
                                    self.complex_types.insert(name, obj);
                                }
                            }
                        }
                    }

                    if local == "simpleType" {
                        if let Some((name, _)) = in_simple_type.take() {
                            let ir_type = if !current_enum_values.is_empty() {
                                IrType::Enum(IrEnum {
                                    name: Some(name.clone()),
                                    values: current_enum_values.clone(),
                                    base_type: Box::new(
                                        current_simple_base.clone().unwrap_or(IrType::String),
                                    ),
                                })
                            } else {
                                current_simple_base.clone().unwrap_or(IrType::String)
                            };
                            self.simple_types.insert(name, ir_type);
                            current_constraints.clear();
                            current_enum_values.clear();
                            current_simple_base = None;
                        }
                    }

                    if depth > 0 {
                        depth -= 1;
                    }
                }

                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(IngestionError::parse(
                        "Xsd",
                        "$",
                        format!("XSD parse error: {}", e),
                    ));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    // ── Pass 2: Build root IrNode ─────────────────────────────────────────────

    fn build_root(&mut self, input: &[u8]) -> Result<IrNode, IngestionError> {
        let mut reader = Reader::from_reader(input);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut root_obj = IrObject::new(Some("schema".to_string()));
        let mut depth = 0usize;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = local_name(e);
                    depth += 1;

                    if local == "element" && depth == 2 {
                        // Top-level element
                        let field = self.element_to_field_resolved(e);
                        root_obj.fields.push(field);
                    }
                }

                Ok(Event::Empty(ref e)) => {
                    let local = local_name(e);
                    if local == "element" && depth == 1 {
                        let field = self.element_to_field_resolved(e);
                        root_obj.fields.push(field);
                    }
                }

                Ok(Event::End(_)) => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }

                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(IngestionError::parse(
                        "Xsd",
                        "$",
                        format!("XSD parse error: {}", e),
                    ));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(IrNode::Object(root_obj))
    }

    fn element_to_field(&self, e: &BytesStart) -> IrField {
        let name = attr_value(e, "name").unwrap_or_else(|| "unknown".to_string());
        let type_ref = attr_value(e, "type");
        let min_occurs = attr_value(e, "minOccurs")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);

        let ir_type = type_ref
            .as_deref()
            .map(xsd_type_to_ir)
            .unwrap_or(IrType::Unknown);

        let mut field = IrField::new(name, ir_type);
        field.nullable = min_occurs == 0;

        if let Some(default) = attr_value(e, "default") {
            field.default_value = Some(serde_json::Value::String(default));
        }

        field
    }

    fn element_to_field_resolved(&self, e: &BytesStart) -> IrField {
        let name = attr_value(e, "name").unwrap_or_else(|| "unknown".to_string());
        let type_ref = attr_value(e, "type");
        let min_occurs = attr_value(e, "minOccurs")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);

        let ir_type = if let Some(type_name) = &type_ref {
            // Try to resolve from collected types first
            let local_type = type_name.split(':').last().unwrap_or(type_name);
            if let Some(complex_obj) = self.complex_types.get(local_type) {
                IrType::Object(Box::new(complex_obj.clone()))
            } else if let Some(simple_type) = self.simple_types.get(local_type) {
                simple_type.clone()
            } else {
                xsd_type_to_ir(type_name)
            }
        } else {
            IrType::Unknown
        };

        let mut field = IrField::new(name, ir_type);
        field.nullable = min_occurs == 0;

        if let Some(default) = attr_value(e, "default") {
            field.default_value = Some(serde_json::Value::String(default));
        }

        field
    }

    fn attribute_to_field(&self, e: &BytesStart) -> IrField {
        let name = attr_value(e, "name").unwrap_or_else(|| "unknown".to_string());
        let type_ref = attr_value(e, "type");
        let use_attr = attr_value(e, "use").unwrap_or_else(|| "optional".to_string());

        let ir_type = type_ref
            .as_deref()
            .map(xsd_type_to_ir)
            .unwrap_or(IrType::String);

        let mut field = IrField::new(format!("@{}", name), ir_type);
        field.nullable = use_attr != "required";
        field.required = use_attr == "required";
        field.metadata.insert(
            "xml_attribute".to_string(),
            serde_json::Value::Bool(true),
        );

        if let Some(default) = attr_value(e, "default") {
            field.default_value = Some(serde_json::Value::String(default));
        }

        field
    }
}

// ── XSD type mapping ──────────────────────────────────────────────────────────

/// Map an XSD built-in type name to IrType.
pub fn xsd_type_to_ir(xsd_type: &str) -> IrType {
    // Strip namespace prefix (xs:, xsd:)
    let local = xsd_type.split(':').last().unwrap_or(xsd_type);

    match local {
        "string" | "token" | "normalizedString" | "Name" | "NCName"
        | "NMTOKEN" | "language" | "anySimpleType" => IrType::String,

        "ID" | "IDREF" | "IDREFS" | "ENTITY" | "ENTITIES" | "NMTOKENS" => {
            // These are string-based with special semantics
            IrType::String
        }

        "integer" | "int" | "long" | "short" | "byte"
        | "nonNegativeInteger" | "positiveInteger"
        | "nonPositiveInteger" | "negativeInteger"
        | "unsignedInt" | "unsignedLong" | "unsignedShort" | "unsignedByte" => IrType::Integer,

        "decimal" | "float" | "double" => IrType::Float,

        "boolean" => IrType::Boolean,

        "date" => IrType::Date,
        "dateTime" => IrType::DateTime,
        "time" => IrType::Time,
        "duration" | "yearMonthDuration" | "dayTimeDuration" => IrType::Duration,

        "base64Binary" | "hexBinary" => IrType::Binary,

        "anyURI" => IrType::Uri,

        "gYear" | "gYearMonth" | "gMonth" | "gMonthDay" | "gDay" => IrType::String,

        _ => IrType::Unknown,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Get the local name (without namespace prefix) from a BytesStart event.
fn local_name(e: &BytesStart) -> String {
    let name_bytes = e.name();
    let raw = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");
    raw.split(':').last().unwrap_or(raw).to_string()
}

/// Get an attribute value by name from a BytesStart event.
fn attr_value(e: &BytesStart, attr_name: &str) -> Option<String> {
    for attr_result in e.attributes() {
        if let Ok(attr) = attr_result {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let local_key = key.split(':').last().unwrap_or(key);
            if local_key == attr_name {
                return std::str::from_utf8(&attr.value)
                    .ok()
                    .map(String::from);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_simple_xsd() {
        let input = br#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="User" type="UserType"/>
  <xs:complexType name="UserType">
    <xs:sequence>
      <xs:element name="id" type="xs:integer"/>
      <xs:element name="email" type="xs:string"/>
      <xs:element name="birthDate" type="xs:date" minOccurs="0"/>
    </xs:sequence>
    <xs:attribute name="active" type="xs:boolean" use="required"/>
  </xs:complexType>
</xs:schema>"#;

        let reader = XsdReader;
        let hint = FormatHint::from_filename("schema.xsd");
        let doc = reader.read(input, &hint).unwrap();

        assert_eq!(doc.source_format, SourceFormat::Xsd);
        if let IrNode::Object(obj) = &doc.root {
            // Should have the User element
            let user_field = obj.fields.iter().find(|f| f.name == "User");
            assert!(user_field.is_some(), "Expected User field, got: {:?}", obj.fields.iter().map(|f| &f.name).collect::<Vec<_>>());
        } else {
            panic!("Expected IrNode::Object");
        }
    }

    #[test]
    fn test_xsd_type_mapping() {
        assert!(matches!(xsd_type_to_ir("xs:string"), IrType::String));
        assert!(matches!(xsd_type_to_ir("xs:integer"), IrType::Integer));
        assert!(matches!(xsd_type_to_ir("xs:decimal"), IrType::Float));
        assert!(matches!(xsd_type_to_ir("xs:boolean"), IrType::Boolean));
        assert!(matches!(xsd_type_to_ir("xs:date"), IrType::Date));
        assert!(matches!(xsd_type_to_ir("xs:dateTime"), IrType::DateTime));
        assert!(matches!(xsd_type_to_ir("xs:anyURI"), IrType::Uri));
        assert!(matches!(xsd_type_to_ir("xs:base64Binary"), IrType::Binary));
    }
}
