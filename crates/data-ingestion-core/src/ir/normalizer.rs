use std::collections::{HashMap, HashSet};

use crate::error::IngestionError;
use crate::ir::model::{IrConstraint, IrDocument, IrField, IrNode, IrType};

/// Post-processes the raw IrDocument produced by readers.
/// Always applied before transformation.
pub struct IrNormalizer;

impl IrNormalizer {
    /// Normalize an IrDocument through all normalization steps.
    pub fn normalize(mut doc: IrDocument) -> Result<IrDocument, IngestionError> {
        // Collect all named type definitions for reference resolution
        let mut type_map: HashMap<String, IrNode> = HashMap::new();
        collect_named_types(&doc.root, &mut type_map);

        // Step 1 — Reference Resolution
        let mut visited: HashSet<String> = HashSet::new();
        doc.root = resolve_references(doc.root, &type_map, &mut visited)?;

        // Step 2 — Type Unification (JSON datasets only)
        // (Type unification is handled during reading for JSON datasets;
        //  the normalizer ensures nullable inference is consistent.)

        // Step 3 — Name Normalization
        doc.root = normalize_names(doc.root);

        // Step 4 — AllOf Flattening
        doc.root = flatten_allof(doc.root);

        // Step 5 — Nullable Inference
        doc.root = infer_nullable(doc.root);

        let _ = doc.source_format; // suppress unused warning
        Ok(doc)
    }
}

// ── Step 1: Reference Resolution ─────────────────────────────────────────────

fn collect_named_types(node: &IrNode, map: &mut HashMap<String, IrNode>) {
    match node {
        IrNode::Object(obj) => {
            if let Some(name) = &obj.name {
                map.insert(name.clone(), node.clone());
            }
            for field in &obj.fields {
                collect_named_types_from_type(&field.ir_type, map);
            }
        }
        IrNode::Array(arr) => {
            collect_named_types(&arr.item_type, map);
        }
        IrNode::Field(f) => {
            collect_named_types_from_type(&f.ir_type, map);
        }
        IrNode::Enum(e) => {
            if let Some(name) = &e.name {
                map.insert(name.clone(), node.clone());
            }
        }
        IrNode::Union(nodes) => {
            for n in nodes {
                collect_named_types(n, map);
            }
        }
        IrNode::Reference(_) => {}
    }
}

fn collect_named_types_from_type(ir_type: &IrType, map: &mut HashMap<String, IrNode>) {
    match ir_type {
        IrType::Object(obj) => {
            if let Some(name) = &obj.name {
                map.insert(name.clone(), IrNode::Object(*obj.clone()));
            }
            for field in &obj.fields {
                collect_named_types_from_type(&field.ir_type, map);
            }
        }
        IrType::Array(arr) => {
            collect_named_types(&arr.item_type, map);
        }
        IrType::Union(types) => {
            for t in types {
                collect_named_types_from_type(t, map);
            }
        }
        _ => {}
    }
}

fn resolve_references(
    node: IrNode,
    type_map: &HashMap<String, IrNode>,
    visited: &mut HashSet<String>,
) -> Result<IrNode, IngestionError> {
    match node {
        IrNode::Reference(ref_name) => {
            if visited.contains(&ref_name) {
                return Err(IngestionError::CircularReference {
                    path: ref_name.clone(),
                });
            }
            match type_map.get(&ref_name) {
                Some(resolved) => {
                    visited.insert(ref_name.clone());
                    let result = resolve_references(resolved.clone(), type_map, visited)?;
                    visited.remove(&ref_name);
                    Ok(result)
                }
                None => Err(IngestionError::UnresolvedReference {
                    reference: ref_name,
                }),
            }
        }
        IrNode::Object(mut obj) => {
            let mut resolved_fields = Vec::new();
            for field in obj.fields {
                let resolved_field = resolve_field_references(field, type_map, visited)?;
                resolved_fields.push(resolved_field);
            }
            obj.fields = resolved_fields;
            Ok(IrNode::Object(obj))
        }
        IrNode::Array(mut arr) => {
            arr.item_type = Box::new(resolve_references(*arr.item_type, type_map, visited)?);
            Ok(IrNode::Array(arr))
        }
        IrNode::Union(nodes) => {
            let mut resolved = Vec::new();
            for n in nodes {
                resolved.push(resolve_references(n, type_map, visited)?);
            }
            Ok(IrNode::Union(resolved))
        }
        other => Ok(other),
    }
}

fn resolve_field_references(
    mut field: IrField,
    type_map: &HashMap<String, IrNode>,
    visited: &mut HashSet<String>,
) -> Result<IrField, IngestionError> {
    field.ir_type = resolve_type_references(field.ir_type, type_map, visited)?;
    Ok(field)
}

fn resolve_type_references(
    ir_type: IrType,
    type_map: &HashMap<String, IrNode>,
    visited: &mut HashSet<String>,
) -> Result<IrType, IngestionError> {
    match ir_type {
        IrType::Object(mut obj) => {
            let mut resolved_fields = Vec::new();
            for field in obj.fields {
                resolved_fields.push(resolve_field_references(field, type_map, visited)?);
            }
            obj.fields = resolved_fields;
            Ok(IrType::Object(obj))
        }
        IrType::Array(mut arr) => {
            arr.item_type = Box::new(resolve_references(*arr.item_type, type_map, visited)?);
            Ok(IrType::Array(arr))
        }
        IrType::Union(types) => {
            let mut resolved = Vec::new();
            for t in types {
                resolved.push(resolve_type_references(t, type_map, visited)?);
            }
            Ok(IrType::Union(resolved))
        }
        other => Ok(other),
    }
}

// ── Step 3: Name Normalization ────────────────────────────────────────────────

fn normalize_names(node: IrNode) -> IrNode {
    match node {
        IrNode::Object(mut obj) => {
            let mut normalized_fields = Vec::new();
            for mut field in obj.fields {
                let original = field.name.clone();
                let normalized = to_snake_case(&original);
                if normalized != original {
                    field
                        .metadata
                        .insert("original_name".to_string(), serde_json::Value::String(original));
                    field.name = normalized;
                }
                field.ir_type = normalize_type_names(field.ir_type);
                normalized_fields.push(field);
            }
            obj.fields = normalized_fields;
            IrNode::Object(obj)
        }
        IrNode::Array(mut arr) => {
            arr.item_type = Box::new(normalize_names(*arr.item_type));
            IrNode::Array(arr)
        }
        IrNode::Union(nodes) => {
            IrNode::Union(nodes.into_iter().map(normalize_names).collect())
        }
        other => other,
    }
}

fn normalize_type_names(ir_type: IrType) -> IrType {
    match ir_type {
        IrType::Object(mut obj) => {
            let mut normalized_fields = Vec::new();
            for mut field in obj.fields {
                let original = field.name.clone();
                let normalized = to_snake_case(&original);
                if normalized != original {
                    field
                        .metadata
                        .insert("original_name".to_string(), serde_json::Value::String(original));
                    field.name = normalized;
                }
                field.ir_type = normalize_type_names(field.ir_type);
                normalized_fields.push(field);
            }
            obj.fields = normalized_fields;
            IrType::Object(obj)
        }
        IrType::Array(mut arr) => {
            arr.item_type = Box::new(normalize_names(*arr.item_type));
            IrType::Array(arr)
        }
        IrType::Union(types) => {
            IrType::Union(types.into_iter().map(normalize_type_names).collect())
        }
        other => other,
    }
}

/// Convert a string to snake_case.
/// Handles: spaces, hyphens, dots, camelCase, PascalCase.
pub fn to_snake_case(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if c == ' ' || c == '-' || c == '.' || c == '_' {
            // Replace separators with underscore (avoid double underscores)
            if !result.ends_with('_') {
                result.push('_');
            }
        } else if c.is_uppercase() {
            // camelCase / PascalCase: insert underscore before uppercase
            // unless it's the first char or previous was also uppercase (acronym)
            let prev_is_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_is_lower = i + 1 < len && chars[i + 1].is_lowercase();
            let prev_is_sep = i > 0
                && (chars[i - 1] == '_'
                    || chars[i - 1] == ' '
                    || chars[i - 1] == '-'
                    || chars[i - 1] == '.');

            if i > 0 && !prev_is_sep && (prev_is_lower || next_is_lower) {
                if !result.ends_with('_') {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c);
        }
    }

    // Remove leading/trailing underscores
    result.trim_matches('_').to_string()
}

// ── Step 4: AllOf Flattening ──────────────────────────────────────────────────

fn flatten_allof(node: IrNode) -> IrNode {
    match node {
        IrNode::Object(mut obj) => {
            let mut flattened_fields = Vec::new();
            for mut field in obj.fields {
                field.constraints = flatten_constraints(field.constraints);
                field.ir_type = flatten_type_allof(field.ir_type);
                flattened_fields.push(field);
            }
            obj.fields = flattened_fields;
            IrNode::Object(obj)
        }
        IrNode::Array(mut arr) => {
            arr.item_type = Box::new(flatten_allof(*arr.item_type));
            IrNode::Array(arr)
        }
        IrNode::Union(nodes) => {
            IrNode::Union(nodes.into_iter().map(flatten_allof).collect())
        }
        other => other,
    }
}

fn flatten_type_allof(ir_type: IrType) -> IrType {
    match ir_type {
        IrType::Object(mut obj) => {
            for field in &mut obj.fields {
                field.constraints = flatten_constraints(std::mem::take(&mut field.constraints));
                field.ir_type = flatten_type_allof(std::mem::replace(&mut field.ir_type, IrType::Unknown));
            }
            IrType::Object(obj)
        }
        IrType::Array(mut arr) => {
            arr.item_type = Box::new(flatten_allof(*arr.item_type));
            IrType::Array(arr)
        }
        IrType::Union(types) => {
            IrType::Union(types.into_iter().map(flatten_type_allof).collect())
        }
        other => other,
    }
}

fn flatten_constraints(constraints: Vec<IrConstraint>) -> Vec<IrConstraint> {
    let mut result = Vec::new();
    for c in constraints {
        match c {
            IrConstraint::AllOf(inner) => {
                let flattened = flatten_constraints(inner);
                for ic in flattened {
                    if !result.iter().any(|existing| constraints_equal(existing, &ic)) {
                        result.push(ic);
                    }
                }
            }
            other => {
                if !result.iter().any(|existing| constraints_equal(existing, &other)) {
                    result.push(other);
                }
            }
        }
    }
    result
}

fn constraints_equal(a: &IrConstraint, b: &IrConstraint) -> bool {
    // Simple structural equality check using debug representation
    format!("{:?}", a) == format!("{:?}", b)
}

// ── Step 5: Nullable Inference ────────────────────────────────────────────────

fn infer_nullable(node: IrNode) -> IrNode {
    match node {
        IrNode::Object(mut obj) => {
            let mut inferred_fields = Vec::new();
            for mut field in obj.fields {
                field = infer_field_nullable(field);
                inferred_fields.push(field);
            }
            obj.fields = inferred_fields;
            IrNode::Object(obj)
        }
        IrNode::Array(mut arr) => {
            arr.item_type = Box::new(infer_nullable(*arr.item_type));
            IrNode::Array(arr)
        }
        IrNode::Union(nodes) => {
            IrNode::Union(nodes.into_iter().map(infer_nullable).collect())
        }
        other => other,
    }
}

fn infer_field_nullable(mut field: IrField) -> IrField {
    field.ir_type = infer_type_nullable(&mut field.nullable, field.ir_type);
    field
}

fn infer_type_nullable(nullable: &mut bool, ir_type: IrType) -> IrType {
    match ir_type {
        IrType::Union(types) => {
            // If Union contains Unknown, remove it and set nullable = true
            let has_unknown = types.iter().any(|t| matches!(t, IrType::Unknown));
            if has_unknown {
                *nullable = true;
                let non_unknown: Vec<IrType> = types
                    .into_iter()
                    .filter(|t| !matches!(t, IrType::Unknown))
                    .collect();
                if non_unknown.len() == 1 {
                    // Collapse single-type union
                    non_unknown.into_iter().next().unwrap_or(IrType::Unknown)
                } else if non_unknown.is_empty() {
                    IrType::Unknown
                } else {
                    IrType::Union(non_unknown)
                }
            } else {
                IrType::Union(types)
            }
        }
        IrType::Object(mut obj) => {
            for field in &mut obj.fields {
                field.ir_type = infer_type_nullable(
                    &mut field.nullable,
                    std::mem::replace(&mut field.ir_type, IrType::Unknown),
                );
            }
            IrType::Object(obj)
        }
        IrType::Array(mut arr) => {
            // Arrays themselves don't get nullable inference from item types
            let _ = arr;
            IrType::Array(arr)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("firstName"), "first_name");
        assert_eq!(to_snake_case("FirstName"), "first_name");
        assert_eq!(to_snake_case("first-name"), "first_name");
        assert_eq!(to_snake_case("first name"), "first_name");
        assert_eq!(to_snake_case("first.name"), "first_name");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("HTTPSRequest"), "https_request");
        assert_eq!(to_snake_case("userID"), "user_id");
    }
}
