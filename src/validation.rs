use serde_json::Value;
use crate::error::TuppError;
use crate::models::TuppData;
use std::path::Path;
use std::fs;

/// Validate a JSON value by attempting to deserialize it into the canonical
/// `TuppData` structure. Any mismatch in shape or types will surface as a
/// descriptive serde error without requiring an external jsonschema crate.
pub fn validate_json(json: &Value) -> Result<(), TuppError> {
    serde_json::from_value::<TuppData>(json.clone())
        .map(|_| ())
        .map_err(|e| TuppError::Validation(format!("Validation failed: {}", e)))
}

pub fn validate_file(path: &Path) -> Result<(), TuppError> {
    let content = fs::read_to_string(path).map_err(TuppError::Io)?;
    let json: Value = serde_json::from_str(&content).map_err(TuppError::Serialization)?;
    validate_json(&json)
}

#[allow(dead_code)]
pub fn validate_data(data: &TuppData) -> Result<(), TuppError> {
    let json_value = serde_json::to_value(data).map_err(TuppError::Serialization)?;
    validate_json(&json_value)
}
