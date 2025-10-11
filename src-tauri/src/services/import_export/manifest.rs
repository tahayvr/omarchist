use serde::{Deserialize, Serialize};

/// Current manifest schema version for `.omarchy` bundles.
pub const MANIFEST_SCHEMA_VERSION: &str = "1.0";

/// Describes the contents of a theme export bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManifest {
    pub schema_version: String,
    pub exporter_version: String,
    pub generated_at: String,
    pub theme_name: String,
    #[serde(default)]
    pub payloads: Vec<ManifestPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ThemeManifest {
    /// Creates a new manifest with required metadata.
    pub fn new(theme_name: impl Into<String>, exporter_version: impl Into<String>) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            exporter_version: exporter_version.into(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            theme_name: theme_name.into(),
            payloads: Vec::new(),
            notes: None,
        }
    }

    /// Adds a payload entry to the manifest.
    pub fn add_payload(&mut self, payload: ManifestPayload) {
        self.payloads.push(payload);
    }
}

/// Individual item tracked inside a theme bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

impl ManifestPayload {
    /// Helper for creating payload entries.
    pub fn new(
        payload_type: impl Into<String>,
        path: impl Into<String>,
        checksum: Option<String>,
    ) -> Self {
        Self {
            payload_type: payload_type.into(),
            path: path.into(),
            checksum,
        }
    }
}
