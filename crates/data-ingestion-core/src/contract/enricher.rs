use crate::contract::model::DataClassification;

// ── PII detection patterns ────────────────────────────────────────────────────

/// Known PII-related substrings (case-insensitive match against field name).
static PII_PATTERNS: &[&str] = &[
    "ssn",
    "social_security",
    "passport",
    "credit_card",
    "card_number",
    "cvv",
    "pin",
    "password",
    "secret",
    "token",
    "api_key",
    "private_key",
    "email",
    "phone",
    "mobile",
    "address",
    "zip",
    "postal",
    "birth_date",
    "dob",
    "age",
    "gender",
    "race",
    "ethnicity",
    "salary",
    "income",
    "tax_id",
    "ein",
    "national_id",
    "ip_address",
    "mac_address",
    "device_id",
    "biometric",
    "first_name",
    "last_name",
    "full_name",
    "username",
];

// ── MetadataEnricher ──────────────────────────────────────────────────────────

/// Enriches contract fields with PII detection and data classification.
pub struct MetadataEnricher;

impl MetadataEnricher {
    /// Returns `true` if the field name matches any known PII pattern
    /// (case-insensitive substring match).
    pub fn detect_pii(field_name: &str) -> bool {
        let lower = field_name.to_lowercase();
        PII_PATTERNS.iter().any(|pattern| lower.contains(pattern))
    }

    /// Returns the appropriate [`DataClassification`] based on field name and
    /// whether the field was detected as PII.
    pub fn classify(field_name: &str, is_pii: bool) -> DataClassification {
        if is_pii {
            // Highly sensitive PII → Restricted
            let lower = field_name.to_lowercase();
            let restricted_patterns = [
                "ssn",
                "social_security",
                "passport",
                "credit_card",
                "card_number",
                "cvv",
                "pin",
                "password",
                "secret",
                "private_key",
                "biometric",
                "tax_id",
                "ein",
                "national_id",
            ];
            if restricted_patterns.iter().any(|p| lower.contains(p)) {
                DataClassification::Restricted
            } else {
                DataClassification::Confidential
            }
        } else {
            DataClassification::Internal
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_detected_email() {
        assert!(MetadataEnricher::detect_pii("email"));
        assert!(MetadataEnricher::detect_pii("user_email"));
        assert!(MetadataEnricher::detect_pii("EMAIL_ADDRESS"));
    }

    #[test]
    fn test_pii_detected_ssn() {
        assert!(MetadataEnricher::detect_pii("ssn"));
        assert!(MetadataEnricher::detect_pii("user_ssn"));
        assert!(MetadataEnricher::detect_pii("SSN_NUMBER"));
    }

    #[test]
    fn test_pii_not_detected_product_id() {
        assert!(!MetadataEnricher::detect_pii("product_id"));
        assert!(!MetadataEnricher::detect_pii("order_count"));
        assert!(!MetadataEnricher::detect_pii("price"));
        assert!(!MetadataEnricher::detect_pii("sku"));
    }

    #[test]
    fn test_classify_pii_restricted() {
        let cls = MetadataEnricher::classify("ssn", true);
        assert_eq!(cls, DataClassification::Restricted);

        let cls = MetadataEnricher::classify("password", true);
        assert_eq!(cls, DataClassification::Restricted);
    }

    #[test]
    fn test_classify_pii_confidential() {
        let cls = MetadataEnricher::classify("email", true);
        assert_eq!(cls, DataClassification::Confidential);

        let cls = MetadataEnricher::classify("first_name", true);
        assert_eq!(cls, DataClassification::Confidential);
    }

    #[test]
    fn test_classify_non_pii_internal() {
        let cls = MetadataEnricher::classify("product_id", false);
        assert_eq!(cls, DataClassification::Internal);
    }
}
