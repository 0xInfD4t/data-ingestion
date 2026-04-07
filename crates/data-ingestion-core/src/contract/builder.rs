use std::collections::HashMap;

use uuid::Uuid;

use crate::contract::enricher::MetadataEnricher;
use crate::contract::model::{
    ContractField, DataClassification, DataContract, FieldConstraint, ForeignKeyRef, LineageInfo,
    LogicalType,
};
use crate::error::TransformError;
use crate::ir::model::{IrConstraint, IrDocument, IrField, IrNode, IrObject, IrType};

// ── ContractBuilderConfig ─────────────────────────────────────────────────────

/// Configuration for the [`ContractBuilder`].
#[derive(Debug, Clone)]
pub struct ContractBuilderConfig {
    /// Semantic version string for the generated contract. Default: `"1.0.0"`.
    pub version: String,
    /// Optional owner identifier.
    pub owner: Option<String>,
    /// Optional domain label.
    pub domain: Option<String>,
    /// When `true`, automatically detect PII fields and set `pii = true`.
    pub enrich_pii: bool,
    /// Default classification applied to non-PII fields.
    pub default_classification: DataClassification,
    /// When `true`, nested objects are preserved as `nested_fields`.
    /// When `false`, nested objects are flattened into the parent field list.
    pub include_nested: bool,
}

impl Default for ContractBuilderConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            owner: None,
            domain: None,
            enrich_pii: true,
            default_classification: DataClassification::Internal,
            include_nested: true,
        }
    }
}

// ── ContractBuilder ───────────────────────────────────────────────────────────

/// Transforms an [`IrDocument`] into a [`DataContract`].
pub struct ContractBuilder {
    config: ContractBuilderConfig,
}

impl ContractBuilder {
    /// Create a new builder with the given configuration.
    pub fn new(config: ContractBuilderConfig) -> Self {
        Self { config }
    }

    /// Build a [`DataContract`] from an [`IrDocument`].
    ///
    /// # Errors
    /// Returns [`TransformError`] if the IR is structurally invalid.
    pub fn build(&self, ir: &IrDocument) -> Result<DataContract, TransformError> {
        log::debug!(
            "ContractBuilder::build: source_format={:?}",
            ir.source_format
        );

        // Generate a stable UUID for this contract
        let id = Uuid::new_v4().to_string();

        // Extract contract name from root node name or fall back to "unnamed"
        let name = ir
            .root
            .name()
            .filter(|n| !n.is_empty())
            .unwrap_or("unnamed")
            .to_string();

        let source_format = ir.source_format.to_string();

        // Extract description from root object if available
        let description = match &ir.root {
            IrNode::Object(obj) => obj.description.clone(),
            _ => None,
        };

        // Build fields from the root node
        let fields = self.convert_node_to_fields(&ir.root, "")?;

        // Build lineage from source hint
        let lineage = ir.source_hint.as_ref().map(|hint| LineageInfo {
            source_system: Some(source_format.clone()),
            source_table: Some(hint.clone()),
            transformation: None,
        });

        let contract = DataContract {
            id,
            name,
            version: self.config.version.clone(),
            description,
            owner: self.config.owner.clone(),
            domain: self.config.domain.clone(),
            source_format,
            fields,
            metadata: HashMap::new(),
            sla: None,
            lineage,
            quality: None,
            created_at: None,
            tags: Vec::new(),
        };

        log::debug!(
            "ContractBuilder::build: produced contract '{}' with {} fields",
            contract.name,
            contract.fields.len()
        );

        Ok(contract)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Convert an [`IrNode`] into a flat list of [`ContractField`]s.
    fn convert_node_to_fields(
        &self,
        node: &IrNode,
        parent_path: &str,
    ) -> Result<Vec<ContractField>, TransformError> {
        match node {
            IrNode::Object(obj) => self.convert_object_fields(obj, parent_path),
            IrNode::Field(field) => {
                let cf = self.convert_field(field, parent_path)?;
                Ok(vec![cf])
            }
            IrNode::Array(arr) => {
                // Treat the array's item type as a single field named after the array
                let name = arr.name.clone().unwrap_or_else(|| "items".to_string());
                let logical_type = LogicalType::Array {
                    item_type: Box::new(self.convert_ir_array_item(&arr.item_type)?),
                };
                let field = ContractField {
                    name: name.clone(),
                    logical_type,
                    physical_type: Some("Array".to_string()),
                    nullable: false,
                    required: false,
                    primary_key: false,
                    foreign_key: None,
                    unique: false,
                    description: None,
                    constraints: Vec::new(),
                    example: None,
                    default_value: None,
                    pii: false,
                    classification: self.config.default_classification.clone(),
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                    nested_fields: Vec::new(),
                };
                Ok(vec![field])
            }
            IrNode::Enum(e) => {
                let name = e.name.clone().unwrap_or_else(|| "enum_field".to_string());
                let allowed: Vec<String> = e
                    .values
                    .iter()
                    .map(|v| v.to_string().trim_matches('"').to_string())
                    .collect();
                let constraints = if allowed.is_empty() {
                    vec![]
                } else {
                    vec![FieldConstraint::AllowedValues(allowed)]
                };
                let field = ContractField {
                    name: name.clone(),
                    logical_type: LogicalType::String,
                    physical_type: Some("Enum".to_string()),
                    nullable: false,
                    required: false,
                    primary_key: false,
                    foreign_key: None,
                    unique: false,
                    description: None,
                    constraints,
                    example: None,
                    default_value: None,
                    pii: false,
                    classification: self.config.default_classification.clone(),
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                    nested_fields: Vec::new(),
                };
                Ok(vec![field])
            }
            IrNode::Union(variants) => {
                // Flatten union variants into individual fields
                let mut fields = Vec::new();
                for variant in variants {
                    fields.extend(self.convert_node_to_fields(variant, parent_path)?);
                }
                Ok(fields)
            }
            IrNode::Reference(r) => {
                // Unresolved reference — emit a placeholder Unknown field
                log::debug!("ContractBuilder: unresolved reference '{}'", r);
                let field = ContractField {
                    name: r.clone(),
                    logical_type: LogicalType::Unknown,
                    physical_type: Some(format!("$ref:{}", r)),
                    nullable: true,
                    required: false,
                    primary_key: false,
                    foreign_key: None,
                    unique: false,
                    description: None,
                    constraints: Vec::new(),
                    example: None,
                    default_value: None,
                    pii: false,
                    classification: self.config.default_classification.clone(),
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                    nested_fields: Vec::new(),
                };
                Ok(vec![field])
            }
        }
    }

    /// Convert all fields of an [`IrObject`] into [`ContractField`]s.
    fn convert_object_fields(
        &self,
        obj: &IrObject,
        parent_path: &str,
    ) -> Result<Vec<ContractField>, TransformError> {
        let mut fields = Vec::new();
        for ir_field in &obj.fields {
            let cf = self.convert_field(ir_field, parent_path)?;
            fields.push(cf);
        }
        Ok(fields)
    }

    /// Convert a single [`IrField`] into a [`ContractField`].
    fn convert_field(
        &self,
        ir_field: &IrField,
        parent_path: &str,
    ) -> Result<ContractField, TransformError> {
        let qualified_name = if parent_path.is_empty() {
            ir_field.name.clone()
        } else {
            format!("{}.{}", parent_path, ir_field.name)
        };

        let physical_type = Some(ir_field.ir_type.to_string());
        let logical_type = self.map_ir_type(&ir_field.ir_type)?;

        // Convert constraints
        let mut constraints = Vec::new();
        let mut primary_key = false;
        let mut unique = false;
        let mut foreign_key: Option<ForeignKeyRef> = None;

        for c in &ir_field.constraints {
            match c {
                IrConstraint::MinLength(v) => {
                    constraints.push(FieldConstraint::MinLength(*v as usize));
                }
                IrConstraint::MaxLength(v) => {
                    constraints.push(FieldConstraint::MaxLength(*v as usize));
                }
                IrConstraint::Pattern(p) => {
                    constraints.push(FieldConstraint::Pattern(p.clone()));
                }
                IrConstraint::Minimum(v) => {
                    constraints.push(FieldConstraint::Minimum(*v));
                }
                IrConstraint::Maximum(v) => {
                    constraints.push(FieldConstraint::Maximum(*v));
                }
                IrConstraint::UniqueItems => {
                    unique = true;
                    constraints.push(FieldConstraint::Unique);
                }
                IrConstraint::Custom(name, value) => {
                    // Handle custom constraints that encode primary_key / foreign_key / unique
                    match name.as_str() {
                        "primary_key" => {
                            primary_key = value.as_bool().unwrap_or(false);
                        }
                        "unique" => {
                            unique = value.as_bool().unwrap_or(false);
                            if unique {
                                constraints.push(FieldConstraint::Unique);
                            }
                        }
                        "foreign_key_table" => {
                            // foreign_key_column must also be present; handled below
                            let _ = value;
                        }
                        _ => {
                            // Ignore other custom constraints silently
                        }
                    }
                }
                // ExclusiveMinimum / ExclusiveMaximum / MultipleOf / MinItems / MaxItems
                // / Const / AllOf / AnyOf — not directly representable; skip
                _ => {}
            }
        }

        // Check for foreign_key encoded as a pair of Custom constraints
        let fk_table = ir_field.constraints.iter().find_map(|c| {
            if let IrConstraint::Custom(k, v) = c {
                if k == "foreign_key_table" {
                    return v.as_str().map(String::from);
                }
            }
            None
        });
        let fk_column = ir_field.constraints.iter().find_map(|c| {
            if let IrConstraint::Custom(k, v) = c {
                if k == "foreign_key_column" {
                    return v.as_str().map(String::from);
                }
            }
            None
        });
        if let (Some(table), Some(column)) = (fk_table, fk_column) {
            foreign_key = Some(ForeignKeyRef { table, column });
        }

        // Add NotNull constraint if not nullable
        if !ir_field.nullable {
            constraints.push(FieldConstraint::NotNull);
        }

        // PII detection
        let pii = if self.config.enrich_pii {
            MetadataEnricher::detect_pii(&ir_field.name)
        } else {
            false
        };

        let classification = if pii {
            MetadataEnricher::classify(&ir_field.name, true)
        } else {
            self.config.default_classification.clone()
        };

        // Convert example values
        let example = ir_field
            .examples
            .first()
            .map(|v| v.to_string().trim_matches('"').to_string());

        // Convert default value
        let default_value = ir_field
            .default_value
            .as_ref()
            .map(|v| v.to_string().trim_matches('"').to_string());

        // Convert metadata (only string values)
        let metadata: HashMap<String, String> = ir_field
            .metadata
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect();

        // Handle nested fields for Object types
        let nested_fields = if self.config.include_nested {
            match &ir_field.ir_type {
                IrType::Object(obj) => self.convert_object_fields(obj, &qualified_name)?,
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        Ok(ContractField {
            name: qualified_name,
            logical_type,
            physical_type,
            nullable: ir_field.nullable,
            required: ir_field.required,
            primary_key,
            foreign_key,
            unique,
            description: ir_field.description.clone(),
            constraints,
            example,
            default_value,
            pii,
            classification,
            tags: ir_field.tags.clone(),
            metadata,
            nested_fields,
        })
    }

    /// Map an [`IrType`] to a [`LogicalType`].
    fn map_ir_type(&self, ir_type: &IrType) -> Result<LogicalType, TransformError> {
        let lt = match ir_type {
            IrType::String   => LogicalType::String,
            IrType::Integer  => LogicalType::Integer,
            IrType::Float    => LogicalType::Double,
            IrType::Boolean  => LogicalType::Boolean,
            IrType::Date     => LogicalType::Date,
            IrType::DateTime => LogicalType::DateTime,
            IrType::Time     => LogicalType::Time,
            IrType::Duration => LogicalType::Duration,
            IrType::Binary   => LogicalType::Binary,
            IrType::Uuid     => LogicalType::Uuid,
            IrType::Uri      => LogicalType::Uri,
            IrType::Email    => LogicalType::Email,
            IrType::Unknown  => LogicalType::Unknown,
            IrType::Object(obj) => {
                let type_name = obj
                    .name
                    .clone()
                    .unwrap_or_else(|| "object".to_string());
                LogicalType::Struct { type_name }
            }
            IrType::Array(arr) => {
                let item_lt = self.convert_ir_array_item(&arr.item_type)?;
                LogicalType::Array {
                    item_type: Box::new(item_lt),
                }
            }
            IrType::Enum(_) => LogicalType::String,
            IrType::Union(variants) => {
                // Use the first non-Unknown variant, or Unknown
                let first = variants
                    .iter()
                    .find(|v| !matches!(v, IrType::Unknown))
                    .unwrap_or(&IrType::Unknown);
                self.map_ir_type(first)?
            }
        };
        Ok(lt)
    }

    /// Convert the item type of an [`IrArray`] node into a [`LogicalType`].
    fn convert_ir_array_item(&self, item_node: &IrNode) -> Result<LogicalType, TransformError> {
        match item_node {
            IrNode::Field(f) => self.map_ir_type(&f.ir_type),
            IrNode::Object(obj) => {
                let type_name = obj.name.clone().unwrap_or_else(|| "object".to_string());
                Ok(LogicalType::Struct { type_name })
            }
            IrNode::Array(arr) => {
                let inner = self.convert_ir_array_item(&arr.item_type)?;
                Ok(LogicalType::Array {
                    item_type: Box::new(inner),
                })
            }
            IrNode::Enum(_) => Ok(LogicalType::String),
            IrNode::Union(variants) => {
                let first = variants.first().map(|v| self.convert_ir_array_item(v));
                match first {
                    Some(Ok(lt)) => Ok(lt),
                    _ => Ok(LogicalType::Unknown),
                }
            }
            IrNode::Reference(_) => Ok(LogicalType::Unknown),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::model::{IrDocument, IrField, IrNode, IrObject, IrType, SourceFormat};

    fn make_simple_ir_doc() -> IrDocument {
        let mut obj = IrObject::new(Some("TestSchema".to_string()));
        obj.fields.push(IrField::new("id", IrType::Integer));
        obj.fields.push(IrField::new("name", IrType::String));
        IrDocument::new(SourceFormat::JsonSchema, None, IrNode::Object(obj))
    }

    #[test]
    fn test_build_produces_two_fields() {
        let config = ContractBuilderConfig::default();
        let builder = ContractBuilder::new(config);
        let ir = make_simple_ir_doc();
        let contract = builder.build(&ir).expect("build should succeed");

        assert_eq!(contract.fields.len(), 2, "expected 2 fields");
        assert_eq!(contract.fields[0].name, "id");
        assert_eq!(contract.fields[1].name, "name");
    }

    #[test]
    fn test_build_contract_name_from_root() {
        let config = ContractBuilderConfig::default();
        let builder = ContractBuilder::new(config);
        let ir = make_simple_ir_doc();
        let contract = builder.build(&ir).expect("build should succeed");

        assert_eq!(contract.name, "TestSchema");
    }

    #[test]
    fn test_build_unnamed_fallback() {
        let config = ContractBuilderConfig::default();
        let builder = ContractBuilder::new(config);
        // Root object with no name
        let obj = IrObject::new(None);
        let ir = IrDocument::new(SourceFormat::JsonSchema, None, IrNode::Object(obj));
        let contract = builder.build(&ir).expect("build should succeed");

        assert_eq!(contract.name, "unnamed");
    }

    #[test]
    fn test_build_version_from_config() {
        let config = ContractBuilderConfig {
            version: "2.3.1".to_string(),
            ..Default::default()
        };
        let builder = ContractBuilder::new(config);
        let ir = make_simple_ir_doc();
        let contract = builder.build(&ir).expect("build should succeed");

        assert_eq!(contract.version, "2.3.1");
    }

    #[test]
    fn test_pii_enrichment_email_field() {
        let config = ContractBuilderConfig {
            enrich_pii: true,
            ..Default::default()
        };
        let builder = ContractBuilder::new(config);
        let mut obj = IrObject::new(Some("Users".to_string()));
        obj.fields.push(IrField::new("email", IrType::String));
        obj.fields.push(IrField::new("product_id", IrType::Integer));
        let ir = IrDocument::new(SourceFormat::JsonSchema, None, IrNode::Object(obj));
        let contract = builder.build(&ir).expect("build should succeed");

        let email_field = contract.fields.iter().find(|f| f.name == "email").unwrap();
        let product_field = contract
            .fields
            .iter()
            .find(|f| f.name == "product_id")
            .unwrap();

        assert!(email_field.pii, "email should be PII");
        assert!(!product_field.pii, "product_id should not be PII");
    }

    #[test]
    fn test_ir_type_mapping() {
        let config = ContractBuilderConfig::default();
        let builder = ContractBuilder::new(config);
        let mut obj = IrObject::new(Some("Types".to_string()));
        obj.fields.push(IrField::new("f_int", IrType::Integer));
        obj.fields.push(IrField::new("f_float", IrType::Float));
        obj.fields.push(IrField::new("f_bool", IrType::Boolean));
        obj.fields.push(IrField::new("f_date", IrType::Date));
        obj.fields.push(IrField::new("f_uuid", IrType::Uuid));
        let ir = IrDocument::new(SourceFormat::JsonSchema, None, IrNode::Object(obj));
        let contract = builder.build(&ir).expect("build should succeed");

        let get = |name: &str| {
            contract
                .fields
                .iter()
                .find(|f| f.name == name)
                .unwrap()
                .logical_type
                .clone()
        };

        assert_eq!(get("f_int"), LogicalType::Integer);
        assert_eq!(get("f_float"), LogicalType::Double);
        assert_eq!(get("f_bool"), LogicalType::Boolean);
        assert_eq!(get("f_date"), LogicalType::Date);
        assert_eq!(get("f_uuid"), LogicalType::Uuid);
    }
}
