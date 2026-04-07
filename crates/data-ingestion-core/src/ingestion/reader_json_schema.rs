use std::collections::HashMap;

use serde_json::Value;

use crate::error::IngestionError;
use crate::ingestion::traits::{FormatHint, FormatReader};
use crate::ir::model::{
    IrArray, IrConstraint, IrDocument, IrEnum, IrField, IrNode, IrObject, IrType, SourceFormat,
};

/// Reads JSON Schema documents (Draft 4, 7, 2019-09, 2020-12).
pub struct JsonSchemaReader;

impl FormatReader for JsonSchemaReader {
    fn can_read(&self, hint: &FormatHint) -> bool {
        if let Some(explicit) = &hint.explicit_format {
            return matches!(explicit, SourceFormat::JsonSchema);
        }
        true
    }

    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
        let schema: Value = serde_json::from_slice(input).map_err(|e| {
            IngestionError::parse_with_source(
                "JsonSchema",
                "$",
                format!("Failed to parse JSON Schema: {}", e),
                Box::new(e),
            )
        })?;

        let mut ctx = SchemaContext::new(&schema);
        let root = ctx.parse_schema(&schema, "root", true)?;

        Ok(IrDocument::new(
            SourceFormat::JsonSchema,
            hint.filename.clone(),
            root,
        ))
    }
}

// ── Schema parsing context ────────────────────────────────────────────────────

struct SchemaContext<'a> {
    /// Root schema (for $ref resolution)
    root: &'a Value,
    /// Collected $defs / definitions
    defs: HashMap<String, &'a Value>,
}

impl<'a> SchemaContext<'a> {
    fn new(root: &'a Value) -> Self {
        let mut defs = HashMap::new();

        // Collect $defs (Draft 2019-09+)
        if let Some(Value::Object(d)) = root.get("$defs") {
            for (k, v) in d {
                defs.insert(format!("#/$defs/{}", k), v);
                defs.insert(k.clone(), v);
            }
        }

        // Collect definitions (Draft 4/7)
        if let Some(Value::Object(d)) = root.get("definitions") {
            for (k, v) in d {
                defs.insert(format!("#/definitions/{}", k), v);
                defs.insert(k.clone(), v);
            }
        }

        Self { root, defs }
    }

    /// Resolve a $ref string to the referenced schema value.
    fn resolve_ref(&self, ref_str: &str) -> Option<&'a Value> {
        // Direct lookup in defs map
        if let Some(v) = self.defs.get(ref_str) {
            return Some(v);
        }

        // JSON Pointer resolution within root
        if ref_str.starts_with('#') {
            let pointer = &ref_str[1..]; // strip leading #
            return self.root.pointer(pointer);
        }

        None
    }

    /// Parse a JSON Schema value into an IrNode.
    fn parse_schema(
        &mut self,
        schema: &'a Value,
        name: &str,
        is_root: bool,
    ) -> Result<IrNode, IngestionError> {
        // Handle $ref
        if let Some(Value::String(ref_str)) = schema.get("$ref") {
            let ref_str = ref_str.clone();
            if let Some(resolved) = self.resolve_ref(&ref_str) {
                // Safety: we need to extend lifetime — this is safe because
                // resolved points into self.root which lives as long as 'a
                let resolved: &'a Value = unsafe { &*(resolved as *const Value) };
                return self.parse_schema(resolved, name, false);
            } else {
                return Ok(IrNode::Reference(ref_str));
            }
        }

        let schema_obj = match schema {
            Value::Object(m) => m,
            Value::Bool(true) => {
                // Schema `true` means any value is valid
                let obj = IrObject::new(Some(name.to_string()));
                return Ok(IrNode::Object(obj));
            }
            Value::Bool(false) => {
                // Schema `false` means no value is valid
                return Ok(IrNode::Field(IrField::new(name, IrType::Unknown)));
            }
            _ => {
                return Err(IngestionError::parse(
                    "JsonSchema",
                    name,
                    "Schema must be an object or boolean",
                ));
            }
        };

        // Handle allOf: merge all sub-schemas into one object
        if let Some(Value::Array(all_of)) = schema_obj.get("allOf") {
            return self.parse_all_of(all_of, name, schema);
        }

        // Handle anyOf / oneOf: produce Union
        if let Some(Value::Array(any_of)) = schema_obj.get("anyOf") {
            return self.parse_any_of(any_of, name, schema);
        }
        if let Some(Value::Array(one_of)) = schema_obj.get("oneOf") {
            return self.parse_any_of(one_of, name, schema);
        }

        // Determine type
        let (ir_type, nullable) = self.extract_type(schema_obj, name)?;

        match &ir_type {
            IrType::Object(_) if is_root || matches!(ir_type, IrType::Object(_)) => {
                // Parse as object with properties
                let mut obj = IrObject::new(Some(name.to_string()));
                obj.description = schema_obj
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                if let Some(Value::String(title)) = schema_obj.get("title") {
                    obj.metadata.insert(
                        "title".to_string(),
                        Value::String(title.clone()),
                    );
                }

                // Collect required fields
                let required_fields: Vec<String> = schema_obj
                    .get("required")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                // Parse properties
                if let Some(Value::Object(props)) = schema_obj.get("properties") {
                    for (prop_name, prop_schema) in props {
                        let child_node = self.parse_schema(prop_schema, prop_name, false)?;
                        let mut field = node_to_field(child_node, prop_name);
                        field.required = required_fields.contains(prop_name);
                        field.nullable = !field.required || nullable;
                        obj.fields.push(field);
                    }
                }

                // Handle additionalProperties
                if let Some(add_props) = schema_obj.get("additionalProperties") {
                    if add_props.is_object() {
                        obj.metadata.insert(
                            "additionalProperties".to_string(),
                            add_props.clone(),
                        );
                    }
                }

                Ok(IrNode::Object(obj))
            }
            _ => {
                // Scalar or array field
                let mut field = IrField::new(name, ir_type);
                field.nullable = nullable;
                self.populate_field_metadata(&mut field, schema_obj);
                Ok(IrNode::Field(field))
            }
        }
    }

    /// Extract IrType and nullable flag from a schema object.
    fn extract_type(
        &mut self,
        schema_obj: &'a serde_json::Map<String, Value>,
        name: &str,
    ) -> Result<(IrType, bool), IngestionError> {
        // Handle enum
        if let Some(Value::Array(enum_vals)) = schema_obj.get("enum") {
            let ir_enum = IrEnum {
                name: Some(name.to_string()),
                values: enum_vals.clone(),
                base_type: Box::new(IrType::String),
            };
            return Ok((IrType::Enum(ir_enum), false));
        }

        // Handle const
        if let Some(const_val) = schema_obj.get("const") {
            let base_type = match const_val {
                Value::String(_) => IrType::String,
                Value::Number(_) => IrType::Integer,
                Value::Bool(_)   => IrType::Boolean,
                _                => IrType::Unknown,
            };
            return Ok((base_type, false));
        }

        let type_val = schema_obj.get("type");
        let format_val = schema_obj.get("format").and_then(|v| v.as_str());

        let (base_type_str, nullable) = match type_val {
            Some(Value::String(t)) => (t.as_str(), false),
            Some(Value::Array(types)) => {
                // ["T", "null"] pattern
                let non_null: Vec<&str> = types
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter(|&s| s != "null")
                    .collect();
                let has_null = types.iter().any(|v| v.as_str() == Some("null"));
                if non_null.len() == 1 {
                    (non_null[0], has_null)
                } else if non_null.is_empty() {
                    return Ok((IrType::Unknown, true));
                } else {
                    // Multiple non-null types → Union
                    let union_types: Vec<IrType> = non_null
                        .iter()
                        .map(|t| json_schema_type_to_ir(t, format_val))
                        .collect();
                    return Ok((IrType::Union(union_types), has_null));
                }
            }
            None => {
                // No type specified — infer from other keywords
                if schema_obj.contains_key("properties") {
                    return Ok((IrType::Object(Box::new(IrObject::new(Some(name.to_string())))), false));
                }
                if schema_obj.contains_key("items") {
                    let item_type = self.parse_array_items(schema_obj, name)?;
                    return Ok((IrType::Array(Box::new(item_type)), false));
                }
                return Ok((IrType::Unknown, false));
            }
            _ => return Ok((IrType::Unknown, false)),
        };

        let ir_type = match base_type_str {
            "object" => {
                IrType::Object(Box::new(IrObject::new(Some(name.to_string()))))
            }
            "array" => {
                let item_type = self.parse_array_items(schema_obj, name)?;
                IrType::Array(Box::new(item_type))
            }
            other => json_schema_type_to_ir(other, format_val),
        };

        Ok((ir_type, nullable))
    }

    /// Parse array items schema.
    fn parse_array_items(
        &mut self,
        schema_obj: &'a serde_json::Map<String, Value>,
        name: &str,
    ) -> Result<IrArray, IngestionError> {
        let item_node = if let Some(items) = schema_obj.get("items") {
            self.parse_schema(items, &format!("{}_item", name), false)?
        } else {
            IrNode::Field(IrField::new(format!("{}_item", name), IrType::Unknown))
        };

        let min_items = schema_obj
            .get("minItems")
            .and_then(|v| v.as_u64());
        let max_items = schema_obj
            .get("maxItems")
            .and_then(|v| v.as_u64());

        Ok(IrArray {
            name: Some(name.to_string()),
            item_type: Box::new(item_node),
            min_items,
            max_items,
        })
    }

    /// Parse allOf: merge all sub-schemas into one object.
    fn parse_all_of(
        &mut self,
        all_of: &'a [Value],
        name: &str,
        parent_schema: &'a Value,
    ) -> Result<IrNode, IngestionError> {
        let mut merged_obj = IrObject::new(Some(name.to_string()));

        // Include properties from parent schema first
        if let Value::Object(parent_obj) = parent_schema {
            if let Some(Value::String(desc)) = parent_obj.get("description") {
                merged_obj.description = Some(desc.clone());
            }
        }

        for sub_schema in all_of {
            let sub_node = self.parse_schema(sub_schema, name, false)?;
            match sub_node {
                IrNode::Object(sub_obj) => {
                    for field in sub_obj.fields {
                        if !merged_obj.fields.iter().any(|f| f.name == field.name) {
                            merged_obj.fields.push(field);
                        }
                    }
                    if merged_obj.description.is_none() {
                        merged_obj.description = sub_obj.description;
                    }
                }
                IrNode::Field(f) => {
                    if !merged_obj.fields.iter().any(|existing| existing.name == f.name) {
                        merged_obj.fields.push(f);
                    }
                }
                _ => {}
            }
        }

        Ok(IrNode::Object(merged_obj))
    }

    /// Parse anyOf / oneOf: produce Union.
    fn parse_any_of(
        &mut self,
        any_of: &'a [Value],
        name: &str,
        _parent_schema: &'a Value,
    ) -> Result<IrNode, IngestionError> {
        // Check for nullable pattern: [{"type": "T"}, {"type": "null"}]
        let has_null_schema = any_of.iter().any(|s| {
            s.get("type").and_then(|v| v.as_str()) == Some("null")
        });

        let non_null_schemas: Vec<&Value> = any_of
            .iter()
            .filter(|s| s.get("type").and_then(|v| v.as_str()) != Some("null"))
            .collect();

        if non_null_schemas.len() == 1 {
            // Single non-null type with optional null → nullable field
            let sub_node = self.parse_schema(non_null_schemas[0], name, false)?;
            let mut field = node_to_field(sub_node, name);
            field.nullable = has_null_schema;
            return Ok(IrNode::Field(field));
        }

        // Multiple types → Union
        let mut union_nodes = Vec::new();
        for sub_schema in any_of {
            if sub_schema.get("type").and_then(|v| v.as_str()) == Some("null") {
                continue; // null handled via nullable flag
            }
            union_nodes.push(self.parse_schema(sub_schema, name, false)?);
        }

        if union_nodes.is_empty() {
            return Ok(IrNode::Field(IrField::new(name, IrType::Unknown)));
        }

        Ok(IrNode::Union(union_nodes))
    }

    /// Populate field metadata from schema keywords.
    fn populate_field_metadata(
        &self,
        field: &mut IrField,
        schema_obj: &serde_json::Map<String, Value>,
    ) {
        field.description = schema_obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        if let Some(Value::String(title)) = schema_obj.get("title") {
            field.metadata.insert("title".to_string(), Value::String(title.clone()));
        }

        if let Some(default) = schema_obj.get("default") {
            field.default_value = Some(default.clone());
        }

        if let Some(Value::Array(examples)) = schema_obj.get("examples") {
            field.examples = examples.clone();
        }

        if let Some(Value::Bool(true)) = schema_obj.get("deprecated") {
            field.deprecated = true;
        }

        // Extract constraints
        if let Some(v) = schema_obj.get("minLength").and_then(|v| v.as_u64()) {
            field.constraints.push(IrConstraint::MinLength(v));
        }
        if let Some(v) = schema_obj.get("maxLength").and_then(|v| v.as_u64()) {
            field.constraints.push(IrConstraint::MaxLength(v));
        }
        if let Some(Value::String(p)) = schema_obj.get("pattern") {
            field.constraints.push(IrConstraint::Pattern(p.clone()));
        }
        if let Some(v) = schema_obj.get("minimum").and_then(|v| v.as_f64()) {
            field.constraints.push(IrConstraint::Minimum(v));
        }
        if let Some(v) = schema_obj.get("maximum").and_then(|v| v.as_f64()) {
            field.constraints.push(IrConstraint::Maximum(v));
        }
        if let Some(v) = schema_obj.get("exclusiveMinimum").and_then(|v| v.as_f64()) {
            field.constraints.push(IrConstraint::ExclusiveMinimum(v));
        }
        if let Some(v) = schema_obj.get("exclusiveMaximum").and_then(|v| v.as_f64()) {
            field.constraints.push(IrConstraint::ExclusiveMaximum(v));
        }
        if let Some(v) = schema_obj.get("multipleOf").and_then(|v| v.as_f64()) {
            field.constraints.push(IrConstraint::MultipleOf(v));
        }
        if let Some(v) = schema_obj.get("minItems").and_then(|v| v.as_u64()) {
            field.constraints.push(IrConstraint::MinItems(v));
        }
        if let Some(v) = schema_obj.get("maxItems").and_then(|v| v.as_u64()) {
            field.constraints.push(IrConstraint::MaxItems(v));
        }
        if let Some(Value::Bool(true)) = schema_obj.get("uniqueItems") {
            field.constraints.push(IrConstraint::UniqueItems);
        }
        if let Some(const_val) = schema_obj.get("const") {
            field.constraints.push(IrConstraint::Const(const_val.clone()));
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Map JSON Schema type string + format to IrType.
fn json_schema_type_to_ir(type_str: &str, format: Option<&str>) -> IrType {
    match type_str {
        "string" => match format {
            Some("date")      => IrType::Date,
            Some("date-time") => IrType::DateTime,
            Some("time")      => IrType::Time,
            Some("duration")  => IrType::Duration,
            Some("uuid")      => IrType::Uuid,
            Some("email")     => IrType::Email,
            Some("uri") | Some("uri-reference") | Some("iri") => IrType::Uri,
            Some("binary") | Some("byte") => IrType::Binary,
            _                 => IrType::String,
        },
        "integer" => IrType::Integer,
        "number"  => IrType::Float,
        "boolean" => IrType::Boolean,
        "null"    => IrType::Unknown,
        _         => IrType::Unknown,
    }
}

/// Convert an IrNode to an IrField (for embedding in objects).
fn node_to_field(node: IrNode, name: &str) -> IrField {
    match node {
        IrNode::Field(f) => f,
        IrNode::Object(obj) => {
            let mut field = IrField::new(name, IrType::Object(Box::new(obj)));
            field
        }
        IrNode::Array(arr) => {
            IrField::new(name, IrType::Array(Box::new(arr)))
        }
        IrNode::Enum(e) => {
            IrField::new(name, IrType::Enum(e))
        }
        IrNode::Union(nodes) => {
            let types: Vec<IrType> = nodes
                .into_iter()
                .map(|n| match n {
                    IrNode::Field(f) => f.ir_type,
                    IrNode::Object(o) => IrType::Object(Box::new(o)),
                    IrNode::Array(a) => IrType::Array(Box::new(a)),
                    IrNode::Enum(e) => IrType::Enum(e),
                    _ => IrType::Unknown,
                })
                .collect();
            IrField::new(name, IrType::Union(types))
        }
        IrNode::Reference(r) => {
            IrField::new(name, IrType::Unknown)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_simple_schema() {
        let input = br#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["id", "email"],
            "properties": {
                "id": {"type": "string", "format": "uuid"},
                "email": {"type": "string", "format": "email", "maxLength": 255},
                "age": {"type": "integer", "minimum": 0, "maximum": 150},
                "score": {"type": ["number", "null"]}
            }
        }"#;

        let reader = JsonSchemaReader;
        let hint = FormatHint::from_filename("schema.json");
        let doc = reader.read(input, &hint).unwrap();

        assert_eq!(doc.source_format, SourceFormat::JsonSchema);
        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.fields.len(), 4);

            let id_field = obj.fields.iter().find(|f| f.name == "id").unwrap();
            assert!(matches!(id_field.ir_type, IrType::Uuid));
            assert!(id_field.required);

            let email_field = obj.fields.iter().find(|f| f.name == "email").unwrap();
            assert!(matches!(email_field.ir_type, IrType::Email));
            assert!(email_field.required);
            assert!(!email_field.constraints.is_empty());

            let score_field = obj.fields.iter().find(|f| f.name == "score").unwrap();
            assert!(score_field.nullable);
        } else {
            panic!("Expected IrNode::Object");
        }
    }

    #[test]
    fn test_read_schema_with_defs() {
        // Use concat! to avoid Rust 2021 reserved prefix issues with $defs/$ref in raw strings
        let input = concat!(
            r#"{"#,
            r#""$schema": "https://json-schema.org/draft/2020-12/schema","#,
            r#""type": "object","#,
            r#""properties": {"address": {"#,
            r#""$ref": "#,
            r##""#/$defs/Address""##,
            r#"}},"#,
            r#""$defs": {"Address": {"type": "object","properties": {"street": {"type": "string"},"city": {"type": "string"}}}}"#,
            r#"}"#
        );

        let reader = JsonSchemaReader;
        let hint = FormatHint::default();
        let doc = reader.read(input.as_bytes(), &hint).unwrap();

        if let IrNode::Object(obj) = &doc.root {
            let addr_field = obj.fields.iter().find(|f| f.name == "address").unwrap();
            assert!(matches!(addr_field.ir_type, IrType::Object(_)));
        } else {
            panic!("Expected IrNode::Object");
        }
    }
}
