use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use crate::contract::model::{ContractField, DataContract, FieldConstraint};
use crate::error::OutputError;

// ── Public API ────────────────────────────────────────────────────────────────

/// Serialize a [`DataContract`] to XML bytes.
pub fn to_xml(contract: &DataContract) -> Result<Vec<u8>, OutputError> {
    log::debug!("output::xml: serializing contract '{}'", contract.name);

    let mut buf = Vec::new();
    let mut writer = Writer::new_with_indent(&mut buf, b' ', 2);

    // XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_err)?;

    // <DataContract id="..." version="..." name="...">
    let mut root = BytesStart::new("DataContract");
    root.push_attribute(("id", contract.id.as_str()));
    root.push_attribute(("version", contract.version.as_str()));
    root.push_attribute(("name", contract.name.as_str()));
    writer.write_event(Event::Start(root)).map_err(xml_err)?;

    // <Description>
    write_optional_element(&mut writer, "Description", contract.description.as_deref())?;

    // <Owner>
    write_optional_element(&mut writer, "Owner", contract.owner.as_deref())?;

    // <Domain>
    write_optional_element(&mut writer, "Domain", contract.domain.as_deref())?;

    // <SourceFormat>
    write_text_element(&mut writer, "SourceFormat", &contract.source_format)?;

    // <Fields>
    writer
        .write_event(Event::Start(BytesStart::new("Fields")))
        .map_err(xml_err)?;
    for field in &contract.fields {
        write_field(&mut writer, field)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("Fields")))
        .map_err(xml_err)?;

    // <Tags>
    if !contract.tags.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("Tags")))
            .map_err(xml_err)?;
        for tag in &contract.tags {
            write_text_element(&mut writer, "Tag", tag)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("Tags")))
            .map_err(xml_err)?;
    }

    // <Metadata>
    if !contract.metadata.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("Metadata")))
            .map_err(xml_err)?;
        for (key, value) in &contract.metadata {
            let mut entry = BytesStart::new("Entry");
            entry.push_attribute(("key", key.as_str()));
            writer
                .write_event(Event::Start(entry))
                .map_err(xml_err)?;
            writer
                .write_event(Event::Text(BytesText::new(value)))
                .map_err(xml_err)?;
            writer
                .write_event(Event::End(BytesEnd::new("Entry")))
                .map_err(xml_err)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("Metadata")))
            .map_err(xml_err)?;
    }

    // </DataContract>
    writer
        .write_event(Event::End(BytesEnd::new("DataContract")))
        .map_err(xml_err)?;

    Ok(buf)
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn xml_err(e: quick_xml::Error) -> OutputError {
    OutputError::SerializationFailed {
        format: "xml".to_string(),
        reason: e.to_string(),
    }
}

fn write_text_element(
    writer: &mut Writer<&mut Vec<u8>>,
    tag: &str,
    text: &str,
) -> Result<(), OutputError> {
    writer
        .write_event(Event::Start(BytesStart::new(tag)))
        .map_err(xml_err)?;
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(xml_err)?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(xml_err)?;
    Ok(())
}

fn write_optional_element(
    writer: &mut Writer<&mut Vec<u8>>,
    tag: &str,
    value: Option<&str>,
) -> Result<(), OutputError> {
    if let Some(v) = value {
        write_text_element(writer, tag, v)?;
    }
    Ok(())
}

fn write_field(
    writer: &mut Writer<&mut Vec<u8>>,
    field: &ContractField,
) -> Result<(), OutputError> {
    let logical_type_str = field.logical_type.to_string();
    let classification_str = field.classification.to_string();

    let mut elem = BytesStart::new("Field");
    elem.push_attribute(("name", field.name.as_str()));
    elem.push_attribute(("logicalType", logical_type_str.as_str()));
    elem.push_attribute(("nullable", if field.nullable { "true" } else { "false" }));
    elem.push_attribute(("required", if field.required { "true" } else { "false" }));
    elem.push_attribute(("primaryKey", if field.primary_key { "true" } else { "false" }));
    elem.push_attribute(("unique", if field.unique { "true" } else { "false" }));
    elem.push_attribute(("pii", if field.pii { "true" } else { "false" }));
    elem.push_attribute(("classification", classification_str.as_str()));

    writer.write_event(Event::Start(elem)).map_err(xml_err)?;

    // <Description>
    write_optional_element(writer, "Description", field.description.as_deref())?;

    // <PhysicalType>
    write_optional_element(writer, "PhysicalType", field.physical_type.as_deref())?;

    // <Example>
    write_optional_element(writer, "Example", field.example.as_deref())?;

    // <DefaultValue>
    write_optional_element(writer, "DefaultValue", field.default_value.as_deref())?;

    // <Constraints>
    if !field.constraints.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("Constraints")))
            .map_err(xml_err)?;
        for constraint in &field.constraints {
            write_constraint(writer, constraint)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("Constraints")))
            .map_err(xml_err)?;
    }

    // <Tags>
    if !field.tags.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("Tags")))
            .map_err(xml_err)?;
        for tag in &field.tags {
            write_text_element(writer, "Tag", tag)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("Tags")))
            .map_err(xml_err)?;
    }

    // <NestedFields>
    if !field.nested_fields.is_empty() {
        writer
            .write_event(Event::Start(BytesStart::new("NestedFields")))
            .map_err(xml_err)?;
        for nested in &field.nested_fields {
            write_field(writer, nested)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("NestedFields")))
            .map_err(xml_err)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("Field")))
        .map_err(xml_err)?;

    Ok(())
}

fn write_constraint(
    writer: &mut Writer<&mut Vec<u8>>,
    constraint: &FieldConstraint,
) -> Result<(), OutputError> {
    let (type_str, value_str) = match constraint {
        FieldConstraint::MinLength(v)      => ("min_length".to_string(), v.to_string()),
        FieldConstraint::MaxLength(v)      => ("max_length".to_string(), v.to_string()),
        FieldConstraint::Pattern(p)        => ("pattern".to_string(), p.clone()),
        FieldConstraint::Minimum(v)        => ("minimum".to_string(), v.to_string()),
        FieldConstraint::Maximum(v)        => ("maximum".to_string(), v.to_string()),
        FieldConstraint::AllowedValues(vs) => ("allowed_values".to_string(), vs.join(",")),
        FieldConstraint::NotNull           => ("not_null".to_string(), "true".to_string()),
        FieldConstraint::Unique            => ("unique".to_string(), "true".to_string()),
    };

    let mut elem = BytesStart::new("Constraint");
    elem.push_attribute(("type", type_str.as_str()));
    elem.push_attribute(("value", value_str.as_str()));
    writer
        .write_event(Event::Empty(elem))
        .map_err(xml_err)?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::model::{ContractField, DataClassification, DataContract, LogicalType};
    use std::collections::HashMap;

    fn make_test_contract() -> DataContract {
        DataContract {
            id: "xml-test-id".to_string(),
            name: "XmlTestContract".to_string(),
            version: "1.0.0".to_string(),
            description: Some("XML test".to_string()),
            owner: Some("owner-team".to_string()),
            domain: Some("finance".to_string()),
            source_format: "JsonSchema".to_string(),
            fields: vec![ContractField {
                name: "amount".to_string(),
                logical_type: LogicalType::Double,
                physical_type: Some("Float".to_string()),
                nullable: false,
                required: true,
                primary_key: false,
                foreign_key: None,
                unique: false,
                description: Some("Transaction amount".to_string()),
                constraints: vec![
                    FieldConstraint::Minimum(0.0),
                    FieldConstraint::Maximum(1_000_000.0),
                ],
                example: Some("99.99".to_string()),
                default_value: None,
                pii: false,
                classification: DataClassification::Internal,
                tags: vec!["financial".to_string()],
                metadata: HashMap::new(),
                nested_fields: Vec::new(),
            }],
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "billing".to_string());
                m
            },
            sla: None,
            lineage: None,
            quality: None,
            created_at: None,
            tags: vec!["finance".to_string()],
        }
    }

    #[test]
    fn test_xml_output_starts_with_declaration() {
        let contract = make_test_contract();
        let bytes = to_xml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(
            s.starts_with("<?xml"),
            "XML should start with declaration, got: {}",
            &s[..50.min(s.len())]
        );
    }

    #[test]
    fn test_xml_output_contains_contract_element() {
        let contract = make_test_contract();
        let bytes = to_xml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("<DataContract"), "XML should contain <DataContract>");
        assert!(s.contains("XmlTestContract"), "XML should contain contract name");
    }

    #[test]
    fn test_xml_output_contains_field() {
        let contract = make_test_contract();
        let bytes = to_xml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("amount"), "XML should contain field name 'amount'");
        assert!(s.contains("<Fields>"), "XML should contain <Fields>");
        assert!(s.contains("<Field"), "XML should contain <Field");
    }

    #[test]
    fn test_xml_output_contains_constraints() {
        let contract = make_test_contract();
        let bytes = to_xml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("minimum"), "XML should contain minimum constraint");
        assert!(s.contains("maximum"), "XML should contain maximum constraint");
    }

    #[test]
    fn test_xml_output_contains_metadata() {
        let contract = make_test_contract();
        let bytes = to_xml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("<Metadata>"), "XML should contain <Metadata>");
        assert!(s.contains("billing"), "XML should contain metadata value");
    }
}
