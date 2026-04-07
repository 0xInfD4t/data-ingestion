use crate::error::DqError;
use crate::expectations::ExpectationSuite;

/// Serialize an ExpectationSuite to YAML bytes.
pub fn to_gx_yaml(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_yaml::to_string(suite)
        .map(|s| s.into_bytes())
        .map_err(|e| DqError::SerializationError(e.to_string()))
}
