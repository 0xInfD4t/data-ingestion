use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::IngestionError;
use crate::ingestion::traits::{FormatHint, FormatReader};
use crate::ingestion::type_inference::infer_type_from_string;
use crate::ir::model::{IrDocument, IrField, IrNode, IrObject, IrType, SourceFormat};

/// Reads raw XML data documents.
pub struct XmlReader;

impl FormatReader for XmlReader {
    fn can_read(&self, hint: &FormatHint) -> bool {
        if let Some(ext) = hint.extension() {
            if ext.eq_ignore_ascii_case("xml") {
                return true;
            }
        }
        if let Some(explicit) = &hint.explicit_format {
            return matches!(explicit, SourceFormat::Xml);
        }
        true
    }

    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
        let root = parse_xml(input)?;
        Ok(IrDocument::new(
            SourceFormat::Xml,
            hint.filename.clone(),
            root,
        ))
    }
}

// ── XML parsing ───────────────────────────────────────────────────────────────

/// Stack frame for building the IrObject tree.
struct StackFrame {
    name: String,
    obj: IrObject,
    text_content: String,
}

fn parse_xml(input: &[u8]) -> Result<IrNode, IngestionError> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<StackFrame> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|err| IngestionError::parse("Xml", "$", format!("Invalid UTF-8 in element name: {}", err)))?
                    .to_string();

                let mut obj = IrObject::new(Some(name.clone()));

                // Parse attributes as fields with "@" prefix
                for attr_result in e.attributes() {
                    let attr = attr_result.map_err(|err| {
                        IngestionError::parse("Xml", &name, format!("Invalid attribute: {}", err))
                    })?;

                    let attr_name = std::str::from_utf8(attr.key.as_ref())
                        .map_err(|err| IngestionError::parse("Xml", &name, format!("Invalid attribute name: {}", err)))?
                        .to_string();

                    let attr_value = std::str::from_utf8(&attr.value)
                        .map_err(|err| IngestionError::parse("Xml", &name, format!("Invalid attribute value: {}", err)))?
                        .to_string();

                    let ir_type = infer_type_from_string(&attr_value);
                    let mut field = IrField::new(format!("@{}", attr_name), ir_type);
                    field.metadata.insert(
                        "xml_attribute".to_string(),
                        serde_json::Value::Bool(true),
                    );
                    field.examples = vec![serde_json::Value::String(attr_value)];
                    obj.fields.push(field);
                }

                stack.push(StackFrame {
                    name,
                    obj,
                    text_content: String::new(),
                });
            }

            Ok(Event::End(_)) => {
                if let Some(frame) = stack.pop() {
                    let mut obj = frame.obj;

                    // If there's text content and no child elements, add _text field
                    let text = frame.text_content.trim().to_string();
                    if !text.is_empty() && obj.fields.iter().all(|f| f.name.starts_with('@')) {
                        let ir_type = infer_type_from_string(&text);
                        let mut text_field = IrField::new("_text", ir_type);
                        text_field.examples = vec![serde_json::Value::String(text)];
                        obj.fields.push(text_field);
                    }

                    if let Some(parent_frame) = stack.last_mut() {
                        // Add this element as a field in the parent
                        let field_name = frame.name.clone();

                        // Check if a field with this name already exists
                        if let Some(existing) = parent_frame.obj.fields.iter_mut().find(|f| f.name == field_name) {
                            // Already exists — upgrade to array or union if needed
                            // For simplicity, keep the first occurrence
                            let _ = existing;
                        } else {
                            let ir_type = if obj.fields.is_empty() {
                                IrType::String // empty element
                            } else {
                                IrType::Object(Box::new(obj))
                            };
                            let field = IrField::new(field_name, ir_type);
                            parent_frame.obj.fields.push(field);
                        }
                    } else {
                        // This is the root element
                        return Ok(IrNode::Object(obj));
                    }
                }
            }

            Ok(Event::Empty(e)) => {
                // Self-closing element: <Element attr="val"/>
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|err| IngestionError::parse("Xml", "$", format!("Invalid UTF-8 in element name: {}", err)))?
                    .to_string();

                let mut obj = IrObject::new(Some(name.clone()));

                for attr_result in e.attributes() {
                    let attr = attr_result.map_err(|err| {
                        IngestionError::parse("Xml", &name, format!("Invalid attribute: {}", err))
                    })?;

                    let attr_name = std::str::from_utf8(attr.key.as_ref())
                        .map_err(|err| IngestionError::parse("Xml", &name, format!("Invalid attribute name: {}", err)))?
                        .to_string();

                    let attr_value = std::str::from_utf8(&attr.value)
                        .map_err(|err| IngestionError::parse("Xml", &name, format!("Invalid attribute value: {}", err)))?
                        .to_string();

                    let ir_type = infer_type_from_string(&attr_value);
                    let mut field = IrField::new(format!("@{}", attr_name), ir_type);
                    field.metadata.insert(
                        "xml_attribute".to_string(),
                        serde_json::Value::Bool(true),
                    );
                    field.examples = vec![serde_json::Value::String(attr_value)];
                    obj.fields.push(field);
                }

                if let Some(parent_frame) = stack.last_mut() {
                    let ir_type = if obj.fields.is_empty() {
                        IrType::String
                    } else {
                        IrType::Object(Box::new(obj))
                    };
                    let field = IrField::new(name, ir_type);
                    parent_frame.obj.fields.push(field);
                }
            }

            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|err| {
                    IngestionError::parse("Xml", "$", format!("Invalid text content: {}", err))
                })?;
                if let Some(frame) = stack.last_mut() {
                    frame.text_content.push_str(&text);
                }
            }

            Ok(Event::CData(e)) => {
                let text = std::str::from_utf8(&e).unwrap_or("").to_string();
                if let Some(frame) = stack.last_mut() {
                    frame.text_content.push_str(&text);
                }
            }

            Ok(Event::Eof) => {
                // If stack is not empty, return the last frame's object
                if let Some(frame) = stack.pop() {
                    return Ok(IrNode::Object(frame.obj));
                }
                break;
            }

            Ok(_) => {} // PI, Comment, Decl — ignore

            Err(e) => {
                return Err(IngestionError::parse(
                    "Xml",
                    "$",
                    format!("XML parse error: {}", e),
                ));
            }
        }
        buf.clear();
    }

    // Empty document
    Ok(IrNode::Object(IrObject::new(Some("root".to_string()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_simple_xml() {
        let input = br#"<?xml version="1.0"?>
<User id="123" active="true">
  <Name>Alice</Name>
  <Email>alice@example.com</Email>
</User>"#;

        let reader = XmlReader;
        let hint = FormatHint::from_filename("data.xml");
        let doc = reader.read(input, &hint).unwrap();

        assert_eq!(doc.source_format, SourceFormat::Xml);
        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.name.as_deref(), Some("User"));

            // Attributes — XML attribute values are text; type is inferred from content.
            // "123" parses as Integer via numeric detection in infer_type_from_string.
            let id_field = obj.fields.iter().find(|f| f.name == "@id").unwrap();
            assert!(matches!(id_field.ir_type, IrType::Integer));
            assert!(id_field.metadata.get("xml_attribute").is_some());

            // Child elements
            let email_field = obj.fields.iter().find(|f| f.name == "Email");
            assert!(email_field.is_some());
        } else {
            panic!("Expected IrNode::Object");
        }
    }

    #[test]
    fn test_read_xml_with_text_content() {
        let input = br#"<root><value>42</value></root>"#;
        let reader = XmlReader;
        let hint = FormatHint::default();
        let doc = reader.read(input, &hint).unwrap();

        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.name.as_deref(), Some("root"));
        } else {
            panic!("Expected IrNode::Object");
        }
    }
}
