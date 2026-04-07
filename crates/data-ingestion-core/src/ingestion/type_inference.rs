use once_cell::sync::Lazy;
use regex::Regex;

use crate::ir::model::IrType;

// ── Compiled regexes (lazy, compiled once) ────────────────────────────────────

static RE_DATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("invalid DATE regex")
});

static RE_DATETIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:?\d{2})?$",
    )
    .expect("invalid DATETIME regex")
});

static RE_UUID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
    )
    .expect("invalid UUID regex")
});

static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$")
        .expect("invalid EMAIL regex")
});

static RE_URI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9+\-.]*://\S+$").expect("invalid URI regex")
});

// ── Public API ────────────────────────────────────────────────────────────────

/// Infer an IrType from a JSON value.
pub fn infer_type_from_json(value: &serde_json::Value) -> IrType {
    match value {
        serde_json::Value::Null    => IrType::Unknown,
        serde_json::Value::Bool(_) => IrType::Boolean,
        serde_json::Value::Number(n) => {
            if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                IrType::Float
            } else {
                IrType::Integer
            }
        }
        serde_json::Value::String(s) => infer_type_from_string(s),
        serde_json::Value::Array(_)  => IrType::Array(Box::new(crate::ir::model::IrArray {
            name: None,
            item_type: Box::new(crate::ir::model::IrNode::Field(
                crate::ir::model::IrField::new("item", IrType::Unknown),
            )),
            min_items: None,
            max_items: None,
        })),
        serde_json::Value::Object(_) => IrType::Object(Box::new(crate::ir::model::IrObject::new(None))),
    }
}

/// Infer an IrType from a string value using pattern matching.
pub fn infer_type_from_string(s: &str) -> IrType {
    if RE_DATETIME.is_match(s) {
        return IrType::DateTime;
    }
    if RE_DATE.is_match(s) {
        return IrType::Date;
    }
    if RE_UUID.is_match(s) {
        return IrType::Uuid;
    }
    if RE_EMAIL.is_match(s) {
        return IrType::Email;
    }
    if RE_URI.is_match(s) {
        return IrType::Uri;
    }
    // Numeric detection (for CSV raw data where all values are strings)
    if let Ok(i) = s.parse::<i64>() {
        let _ = i;
        return IrType::Integer;
    }
    if let Ok(f) = s.parse::<f64>() {
        let _ = f;
        return IrType::Float;
    }
    // Boolean detection
    match s.to_lowercase().as_str() {
        "true" | "false" | "yes" | "no" => return IrType::Boolean,
        _ => {}
    }
    IrType::String
}

/// Infer an IrType from a raw string (e.g. from CSV/YAML type columns).
pub fn infer_type_from_type_string(s: &str) -> IrType {
    match s.trim().to_lowercase().as_str() {
        "string" | "str" | "text" | "varchar" | "char" | "nvarchar" => IrType::String,
        "integer" | "int" | "int32" | "int64" | "long" | "bigint" | "smallint" | "tinyint" => IrType::Integer,
        "float" | "double" | "decimal" | "numeric" | "real" | "number" | "float32" | "float64" => IrType::Float,
        "boolean" | "bool" | "bit" => IrType::Boolean,
        "date" => IrType::Date,
        "datetime" | "timestamp" | "date_time" | "date-time" => IrType::DateTime,
        "time" => IrType::Time,
        "duration" | "interval" => IrType::Duration,
        "binary" | "bytes" | "blob" | "base64" => IrType::Binary,
        "uuid" | "guid" => IrType::Uuid,
        "uri" | "url" | "link" => IrType::Uri,
        "email" => IrType::Email,
        _ => IrType::Unknown,
    }
}

/// Unify a list of observed types into a single IrType.
/// Used for multi-row type inference in JSON datasets.
pub fn unify_types(types: Vec<IrType>) -> (IrType, bool) {
    if types.is_empty() {
        return (IrType::Unknown, true);
    }

    let null_count = types.iter().filter(|t| matches!(t, IrType::Unknown)).count();
    let non_null: Vec<&IrType> = types.iter().filter(|t| !matches!(t, IrType::Unknown)).collect();

    let nullable = null_count > 0;

    if non_null.is_empty() {
        return (IrType::Unknown, true);
    }

    // Collect unique non-null types
    let mut unique_types: Vec<IrType> = Vec::new();
    for t in &non_null {
        let t_str = format!("{:?}", t);
        if !unique_types.iter().any(|u| format!("{:?}", u) == t_str) {
            unique_types.push((*t).clone());
        }
    }

    if unique_types.len() == 1 {
        return (unique_types.into_iter().next().unwrap(), nullable);
    }

    // Multiple types: check if one dominates (>50% of non-null values)
    let non_null_count = non_null.len();
    for candidate in &unique_types {
        let candidate_str = format!("{:?}", candidate);
        let count = non_null.iter().filter(|t| format!("{:?}", t) == candidate_str).count();
        if count as f64 / non_null_count as f64 > 0.5 {
            return (candidate.clone(), nullable);
        }
    }

    // No dominant type: return Union
    (IrType::Union(unique_types), nullable)
}

/// Check if a serde_json Number is an integer (no fractional part).
pub fn json_number_is_integer(n: &serde_json::Number) -> bool {
    if let Some(i) = n.as_i64() {
        let _ = i;
        return true;
    }
    if let Some(u) = n.as_u64() {
        let _ = u;
        return true;
    }
    if let Some(f) = n.as_f64() {
        return f.fract() == 0.0;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_date() {
        assert!(matches!(infer_type_from_string("2024-01-15"), IrType::Date));
    }

    #[test]
    fn test_infer_datetime() {
        assert!(matches!(infer_type_from_string("2024-01-15T10:30:00Z"), IrType::DateTime));
        assert!(matches!(infer_type_from_string("2024-01-15T10:30:00+05:00"), IrType::DateTime));
    }

    #[test]
    fn test_infer_uuid() {
        assert!(matches!(
            infer_type_from_string("550e8400-e29b-41d4-a716-446655440000"),
            IrType::Uuid
        ));
    }

    #[test]
    fn test_infer_email() {
        assert!(matches!(infer_type_from_string("user@example.com"), IrType::Email));
    }

    #[test]
    fn test_infer_uri() {
        assert!(matches!(infer_type_from_string("https://example.com/path"), IrType::Uri));
    }

    #[test]
    fn test_infer_string() {
        assert!(matches!(infer_type_from_string("hello world"), IrType::String));
    }

    #[test]
    fn test_unify_types_single() {
        let types = vec![IrType::Integer, IrType::Integer, IrType::Unknown];
        let (unified, nullable) = unify_types(types);
        assert!(matches!(unified, IrType::Integer));
        assert!(nullable);
    }

    #[test]
    fn test_unify_types_union() {
        let types = vec![IrType::Integer, IrType::String];
        let (unified, nullable) = unify_types(types);
        assert!(matches!(unified, IrType::Union(_)));
        assert!(!nullable);
    }
}
