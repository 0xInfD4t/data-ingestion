use super::ExpectationSuite;
use crate::error::DqError;

/// Serialize an ExpectationSuite to GX 1.x-compatible JSON bytes.
pub fn to_gx_json(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_json::to_vec_pretty(suite)
        .map_err(|e| DqError::SerializationError(e.to_string()))
}

/// Serialize an ExpectationSuite to YAML bytes.
pub fn to_gx_yaml(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_yaml::to_string(suite)
        .map(|s| s.into_bytes())
        .map_err(|e| DqError::SerializationError(e.to_string()))
}
