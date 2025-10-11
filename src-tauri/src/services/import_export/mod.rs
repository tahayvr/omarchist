mod manifest;

use crate::services::themes::{CustomThemeService, ThemeValidator};
use crate::types::ThemeError;
use manifest::{ManifestPayload, ThemeManifest, MANIFEST_SCHEMA_VERSION};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, Write};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use uuid::Uuid;
use zip::read::ZipArchive;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Service for importing and exporting custom themes
pub struct ThemeImportExportService {
    custom_theme_service: CustomThemeService,
    themes_dir: PathBuf,
}

/// Result of a theme import operation
#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub theme_name: String,
    pub message: String,
    pub conflict: Option<ConflictInfo>,
}

/// Information about a naming conflict during import
#[derive(Debug, serde::Serialize, Clone)]
pub struct ConflictInfo {
    pub existing_theme: String,
    pub suggested_name: String,
}

/// Result of theme validation
#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub theme_name: Option<String>,
}

impl ThemeImportExportService {
    /// Create a new import/export service
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let custom_theme_service = CustomThemeService::new(app_handle)?;

        let home_dir =
            dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
        let themes_dir = home_dir.join(".config").join("omarchy").join("themes");

        Ok(Self {
            custom_theme_service,
            themes_dir,
        })
    }

    /// Export a custom theme to a bundle or legacy JSON file
    pub fn export_theme(
        &self,
        theme_name: &str,
        destination: PathBuf,
    ) -> Result<PathBuf, ThemeError> {
        let mut export_path = destination;
        let ext = export_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        if matches!(ext.as_deref(), Some("json")) {
            return self.export_theme_as_json(theme_name, export_path);
        }

        if !matches!(ext.as_deref(), Some("omarchy")) {
            export_path.set_extension("omarchy");
        }

        self.export_theme_as_bundle(theme_name, export_path)
    }

    fn export_theme_as_json(
        &self,
        theme_name: &str,
        mut export_path: PathBuf,
    ) -> Result<PathBuf, ThemeError> {
        let theme = self
            .custom_theme_service
            .get_theme(theme_name)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to get theme: {}", e)))?;

        if export_path.extension().is_none() {
            export_path.set_extension("json");
        }

        let theme_json = serde_json::to_string_pretty(&theme)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to serialize theme: {}", e)))?;

        fs::write(&export_path, theme_json)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to write file: {}", e)))?;

        log::info!(
            "Exported theme '{}' to {}",
            theme_name,
            export_path.display()
        );

        Ok(export_path)
    }

    fn export_theme_as_bundle(
        &self,
        theme_name: &str,
        export_path: PathBuf,
    ) -> Result<PathBuf, ThemeError> {
        let theme = self
            .custom_theme_service
            .get_theme(theme_name)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to get theme: {}", e)))?;

        let sanitized_name = CustomThemeService::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        if !theme_dir.exists() {
            return Err(ThemeError::ExportFailed(format!(
                "Theme directory not found for '{}'",
                theme_name
            )));
        }

        let temp_dir = Self::create_temp_dir("omarchist-export").map_err(|e| {
            ThemeError::ExportFailed(format!("Failed to create bundle workspace: {}", e))
        })?;

        let bundle_result = (|| -> Result<PathBuf, ThemeError> {
            let bundle_root = temp_dir.as_path();

            let theme_json_path = bundle_root.join("theme.json");
            fs::write(
                &theme_json_path,
                serde_json::to_string_pretty(&theme).map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to serialize theme metadata: {}", e))
                })?,
            )
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to write theme.json: {}", e)))?;

            self.copy_backgrounds_for_export(&theme_dir, bundle_root)?;
            self.copy_configs_for_export(&theme_dir, bundle_root, &sanitized_name)?;
            self.copy_media_for_export(&theme_dir, bundle_root)?;

            let mut manifest = ThemeManifest::new(theme.name.clone(), env!("CARGO_PKG_VERSION"));

            let mut files = Self::collect_relative_files(bundle_root)?;
            files.sort();

            for rel_path in files {
                let absolute_path = bundle_root.join(&rel_path);
                let checksum = Self::compute_sha256_hex(&absolute_path).map_err(|e| {
                    ThemeError::ExportFailed(format!(
                        "Failed to hash bundle asset '{}': {}",
                        rel_path.display(),
                        e
                    ))
                })?;

                let normalized = Self::normalize_path(&rel_path);
                let payload_type = if normalized == "theme.json" {
                    "theme"
                } else if normalized.starts_with("backgrounds/") {
                    "background"
                } else if normalized.starts_with("configs/") {
                    "config"
                } else if normalized.starts_with("media/") {
                    "media"
                } else {
                    "asset"
                };

                manifest.add_payload(ManifestPayload::new(
                    payload_type.to_string(),
                    normalized,
                    Some(checksum),
                ));
            }

            let manifest_path = bundle_root.join("manifest.json");
            fs::write(
                &manifest_path,
                serde_json::to_string_pretty(&manifest).map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to serialize manifest: {}", e))
                })?,
            )
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to write manifest: {}", e)))?;

            self.write_bundle_archive(bundle_root, &export_path)?;

            log::info!(
                "Exported theme '{}' to {}",
                theme_name,
                export_path.display()
            );

            Ok(export_path)
        })();

        if let Err(e) = fs::remove_dir_all(&temp_dir) {
            log::warn!(
                "Failed to clean up temporary export directory {}: {}",
                temp_dir.display(),
                e
            );
        }

        bundle_result
    }

    /// Import a theme from a JSON file
    pub fn import_theme_from_file(
        &self,
        file_path: &Path,
        rename_on_conflict: bool,
    ) -> Result<ImportResult, ThemeError> {
        let ext = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        if matches!(ext.as_deref(), Some("json")) {
            return self.import_json_theme(file_path, rename_on_conflict);
        }

        let looks_like_archive =
            matches!(ext.as_deref(), Some("omarchy" | "zip")) || self.is_zip_archive(file_path)?;

        if looks_like_archive {
            return self.import_bundle_theme(file_path, rename_on_conflict);
        }

        self.import_json_theme(file_path, rename_on_conflict)
    }

    /// Validate a theme file without importing it
    pub fn validate_theme_file(&self, file_path: &Path) -> ValidationResult {
        let ext = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        if matches!(ext.as_deref(), Some("json")) {
            return self.validate_json_theme(file_path);
        }

        match self.is_zip_archive(file_path) {
            Ok(true) => self.validate_bundle_theme(file_path),
            Ok(false) => self.validate_json_theme(file_path),
            Err(err) => ValidationResult {
                valid: false,
                errors: vec![format!("Failed to read file: {}", err)],
                theme_name: None,
            },
        }
    }

    fn validate_json_theme(&self, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();
        let mut theme_name = None;

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Failed to read file: {}", e));
                return ValidationResult {
                    valid: false,
                    errors,
                    theme_name: None,
                };
            },
        };

        let theme_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("Invalid JSON format: {}", e));
                return ValidationResult {
                    valid: false,
                    errors,
                    theme_name: None,
                };
            },
        };

        if !ThemeValidator::is_omarchist_theme(&theme_value) {
            errors.push("Not an Omarchist custom theme".to_string());
            return ValidationResult {
                valid: false,
                errors,
                theme_name: None,
            };
        }

        if let Some(name) = theme_value.get("name").and_then(|n| n.as_str()) {
            theme_name = Some(name.to_string());
        }

        if let Err(e) = ThemeValidator::validate_theme(&theme_value) {
            errors.push(format!("Validation error: {}", e));
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            theme_name,
        }
    }

    fn validate_bundle_theme(&self, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();
        let mut theme_name = None;

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                errors.push(format!("Failed to open file: {}", e));
                return ValidationResult {
                    valid: false,
                    errors,
                    theme_name: None,
                };
            },
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(archive) => archive,
            Err(e) => {
                errors.push(format!("Invalid bundle: {}", e));
                return ValidationResult {
                    valid: false,
                    errors,
                    theme_name: None,
                };
            },
        };

        match archive.by_name("manifest.json") {
            Ok(mut manifest_file) => {
                let mut manifest_content = String::new();
                if let Err(e) = manifest_file.read_to_string(&mut manifest_content) {
                    errors.push(format!("Failed to read manifest: {}", e));
                } else if let Err(e) = serde_json::from_str::<ThemeManifest>(&manifest_content) {
                    errors.push(format!("Invalid manifest: {}", e));
                }
            },
            Err(_) => errors.push("Bundle missing manifest.json".to_string()),
        }

        match archive.by_name("theme.json") {
            Ok(mut theme_file) => {
                let mut content = String::new();
                if let Err(e) = theme_file.read_to_string(&mut content) {
                    errors.push(format!("Failed to read theme.json: {}", e));
                } else {
                    match serde_json::from_str::<Value>(&content) {
                        Ok(theme_value) => {
                            if let Some(name) = theme_value.get("name").and_then(|n| n.as_str()) {
                                theme_name = Some(name.to_string());
                            }

                            if let Err(e) = ThemeValidator::validate_theme(&theme_value) {
                                errors.push(format!("Validation error: {}", e));
                            }
                        },
                        Err(e) => errors.push(format!("Invalid theme JSON: {}", e)),
                    }
                }
            },
            Err(_) => errors.push("Bundle missing theme.json".to_string()),
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            theme_name,
        }
    }

    /// Check if a theme with the given name exists
    fn theme_exists(&self, name: &str) -> bool {
        self.custom_theme_service.get_theme(name).is_ok()
    }

    /// Generate a unique theme name by appending a number
    fn generate_unique_name(&self, base_name: &str) -> String {
        let mut counter = 1;
        let mut unique_name = format!("{} ({})", base_name, counter);

        while self.theme_exists(&unique_name) {
            counter += 1;
            unique_name = format!("{} ({})", base_name, counter);
        }

        unique_name
    }

    fn import_json_theme(
        &self,
        file_path: &Path,
        rename_on_conflict: bool,
    ) -> Result<ImportResult, ThemeError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| ThemeError::ImportFailed(format!("Failed to read file: {}", e)))?;

        let mut theme_value: Value = serde_json::from_str(&content)
            .map_err(|e| ThemeError::ImportFailed(format!("Invalid JSON: {}", e)))?;

        if !ThemeValidator::is_omarchist_theme(&theme_value) {
            return Err(ThemeError::ImportFailed(
                "This file does not appear to be an Omarchist custom theme".to_string(),
            ));
        }

        ThemeValidator::validate_theme(&theme_value)?;
        ThemeValidator::sanitize_theme(&mut theme_value)?;

        let original_name = theme_value
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| ThemeError::ImportFailed("Missing theme name".to_string()))?
            .to_string();

        let theme_name =
            match self.resolve_theme_name(&mut theme_value, original_name, rename_on_conflict) {
                Ok(name) => name,
                Err(conflict) => {
                    let existing = conflict.existing_theme.clone();
                    return Ok(ImportResult {
                        success: false,
                        theme_name: existing,
                        message: "Theme already exists".to_string(),
                        conflict: Some(conflict),
                    });
                },
            };

        self.custom_theme_service
            .create_theme_advanced(theme_name.clone(), theme_value.clone())
            .map_err(|e| ThemeError::ImportFailed(format!("Failed to create theme: {}", e)))?;

        log::info!("Successfully imported theme '{}'", theme_name);

        Ok(ImportResult {
            success: true,
            theme_name: theme_name.clone(),
            message: format!("Successfully imported theme '{}'", theme_name),
            conflict: None,
        })
    }

    fn import_bundle_theme(
        &self,
        file_path: &Path,
        rename_on_conflict: bool,
    ) -> Result<ImportResult, ThemeError> {
        let temp_dir = Self::create_temp_dir("omarchist-import").map_err(|e| {
            ThemeError::ImportFailed(format!("Failed to create extraction directory: {}", e))
        })?;

        let result = (|| -> Result<ImportResult, ThemeError> {
            self.extract_bundle(file_path, &temp_dir)?;

            let manifest = self.load_manifest(&temp_dir)?;
            self.verify_bundle_payloads(&manifest, &temp_dir)?;

            let mut theme_value = self.load_bundle_theme(&temp_dir)?;

            if theme_value.get("name").is_none() {
                theme_value.as_object_mut().map(|obj| {
                    obj.insert(
                        "name".to_string(),
                        Value::String(manifest.theme_name.clone()),
                    )
                });
            }

            ThemeValidator::validate_theme(&theme_value)?;
            ThemeValidator::sanitize_theme(&mut theme_value)?;

            let original_name = theme_value
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(&manifest.theme_name)
                .to_string();

            let theme_name = match self.resolve_theme_name(
                &mut theme_value,
                original_name,
                rename_on_conflict,
            ) {
                Ok(name) => name,
                Err(conflict) => {
                    let existing = conflict.existing_theme.clone();
                    return Ok(ImportResult {
                        success: false,
                        theme_name: existing,
                        message: "Theme already exists".to_string(),
                        conflict: Some(conflict),
                    });
                },
            };

            self.custom_theme_service
                .create_theme_advanced(theme_name.clone(), theme_value.clone())
                .map_err(|e| ThemeError::ImportFailed(format!("Failed to create theme: {}", e)))?;

            let sanitized_name = CustomThemeService::sanitize_name(&theme_name);
            let theme_dir = self.themes_dir.join(&sanitized_name);

            self.copy_bundle_assets(&manifest, &temp_dir, &theme_dir)?;
            self.restore_theme_metadata(&theme_dir, &sanitized_name, &theme_value, &theme_name)?;

            log::info!("Successfully imported theme '{}' from bundle", theme_name);

            Ok(ImportResult {
                success: true,
                theme_name: theme_name.clone(),
                message: format!("Successfully imported theme '{}'", theme_name),
                conflict: None,
            })
        })();

        if let Err(e) = fs::remove_dir_all(&temp_dir) {
            log::warn!(
                "Failed to clean up temporary import directory {}: {}",
                temp_dir.display(),
                e
            );
        }

        result
    }

    fn resolve_theme_name(
        &self,
        theme_value: &mut Value,
        original_name: String,
        rename_on_conflict: bool,
    ) -> Result<String, ConflictInfo> {
        if self.theme_exists(&original_name) {
            if rename_on_conflict {
                let unique_name = self.generate_unique_name(&original_name);
                if let Some(obj) = theme_value.as_object_mut() {
                    obj.insert("name".to_string(), Value::String(unique_name.clone()));
                }

                log::info!(
                    "Renamed theme from '{}' to '{}' due to conflict",
                    original_name,
                    unique_name
                );

                Ok(unique_name)
            } else {
                Err(ConflictInfo {
                    existing_theme: original_name.clone(),
                    suggested_name: self.generate_unique_name(&original_name),
                })
            }
        } else {
            Ok(original_name)
        }
    }

    fn extract_bundle(&self, file_path: &Path, destination: &Path) -> Result<(), ThemeError> {
        let file = File::open(file_path)
            .map_err(|e| ThemeError::ImportFailed(format!("Failed to open bundle: {}", e)))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| ThemeError::ImportFailed(format!("Invalid bundle archive: {}", e)))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                ThemeError::ImportFailed(format!("Failed to read bundle entry: {}", e))
            })?;

            let entry_name = entry
                .enclosed_name()
                .ok_or_else(|| {
                    ThemeError::ImportFailed("Bundle contains invalid paths".to_string())
                })?
                .to_owned();

            let out_path = destination.join(&entry_name);

            if entry.name().ends_with('/') {
                fs::create_dir_all(&out_path).map_err(|e| {
                    ThemeError::ImportFailed(format!("Failed to create directory: {}", e))
                })?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        ThemeError::ImportFailed(format!("Failed to create directory: {}", e))
                    })?;
                }

                let mut outfile = File::create(&out_path).map_err(|e| {
                    ThemeError::ImportFailed(format!("Failed to create file: {}", e))
                })?;
                std::io::copy(&mut entry, &mut outfile).map_err(|e| {
                    ThemeError::ImportFailed(format!("Failed to extract bundle entry: {}", e))
                })?;
            }
        }

        Ok(())
    }

    fn load_manifest(&self, bundle_dir: &Path) -> Result<ThemeManifest, ThemeError> {
        let manifest_path = bundle_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(ThemeError::ImportFailed(
                "Bundle is missing manifest.json".to_string(),
            ));
        }

        let manifest: ThemeManifest =
            serde_json::from_str(&fs::read_to_string(&manifest_path).map_err(|e| {
                ThemeError::ImportFailed(format!("Failed to read manifest: {}", e))
            })?)
            .map_err(|e| ThemeError::ImportFailed(format!("Invalid manifest: {}", e)))?;

        if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ThemeError::ImportFailed(format!(
                "Unsupported manifest schema version: {}",
                manifest.schema_version
            )));
        }

        Ok(manifest)
    }

    fn verify_bundle_payloads(
        &self,
        manifest: &ThemeManifest,
        bundle_dir: &Path,
    ) -> Result<(), ThemeError> {
        for payload in &manifest.payloads {
            let path = bundle_dir.join(Path::new(&payload.path));

            if !path.exists() {
                return Err(ThemeError::ImportFailed(format!(
                    "Bundle is missing expected asset: {}",
                    payload.path
                )));
            }

            if let Some(expected) = &payload.checksum {
                let actual = Self::compute_sha256_hex(&path).map_err(|e| {
                    ThemeError::ImportFailed(format!(
                        "Failed to hash bundle asset '{}': {}",
                        payload.path, e
                    ))
                })?;

                if &actual != expected {
                    return Err(ThemeError::ImportFailed(format!(
                        "Checksum mismatch for {}",
                        payload.path
                    )));
                }
            }
        }

        Ok(())
    }

    fn load_bundle_theme(&self, bundle_dir: &Path) -> Result<Value, ThemeError> {
        let theme_path = bundle_dir.join("theme.json");
        if !theme_path.exists() {
            return Err(ThemeError::ImportFailed(
                "Bundle is missing theme.json".to_string(),
            ));
        }

        let content = fs::read_to_string(&theme_path)
            .map_err(|e| ThemeError::ImportFailed(format!("Failed to read theme.json: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| ThemeError::ImportFailed(format!("Invalid theme JSON: {}", e)))
    }

    fn copy_bundle_assets(
        &self,
        manifest: &ThemeManifest,
        bundle_dir: &Path,
        theme_dir: &Path,
    ) -> Result<(), ThemeError> {
        for payload in &manifest.payloads {
            if payload.path == "theme.json" {
                continue;
            }

            let relative = Path::new(&payload.path);
            let src = bundle_dir.join(relative);
            let dest = theme_dir.join(relative);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    ThemeError::ImportFailed(format!("Failed to create directory: {}", e))
                })?;
            }

            fs::copy(&src, &dest).map_err(|e| {
                ThemeError::ImportFailed(format!("Failed to copy asset '{}': {}", payload.path, e))
            })?;

            if payload.payload_type == "config" {
                if let Some(filename) = relative.file_name() {
                    let root_dest = theme_dir.join(filename);
                    if root_dest != dest {
                        if let Some(parent) = root_dest.parent() {
                            fs::create_dir_all(parent).map_err(|e| {
                                ThemeError::ImportFailed(format!(
                                    "Failed to create directory: {}",
                                    e
                                ))
                            })?;
                        }

                        fs::copy(&src, &root_dest).map_err(|e| {
                            ThemeError::ImportFailed(format!(
                                "Failed to copy config '{}': {}",
                                payload.path, e
                            ))
                        })?;
                    }
                }
            }
        }

        Ok(())
    }

    fn restore_theme_metadata(
        &self,
        theme_dir: &Path,
        sanitized_name: &str,
        theme_value: &Value,
        theme_name: &str,
    ) -> Result<(), ThemeError> {
        let mut created_theme = self
            .custom_theme_service
            .get_theme(theme_name)
            .map_err(|e| {
                ThemeError::ImportFailed(format!("Failed to load imported theme: {}", e))
            })?;

        if let Some(created_at) = theme_value.get("created_at").and_then(|v| v.as_str()) {
            created_theme.created_at = created_at.to_string();
        }

        if let Some(modified_at) = theme_value.get("modified_at").and_then(|v| v.as_str()) {
            created_theme.modified_at = modified_at.to_string();
        }

        if let Some(author) = theme_value
            .get("author")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            created_theme.author = Some(author.to_string());
        } else if let Some(author) = theme_value
            .get("metadata")
            .and_then(|meta| meta.get("author"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            created_theme.author = Some(author.to_string());
        }

        if let Some(colors) = theme_value.get("colors") {
            created_theme.colors = serde_json::from_value(colors.clone()).ok();
        }

        let metadata_path = Self::metadata_path(theme_dir, sanitized_name);
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&created_theme).map_err(|e| {
                ThemeError::ImportFailed(format!("Failed to serialize theme metadata: {}", e))
            })?,
        )
        .map_err(|e| ThemeError::ImportFailed(format!("Failed to write theme metadata: {}", e)))
    }

    fn write_bundle_archive(
        &self,
        bundle_root: &Path,
        export_path: &Path,
    ) -> Result<(), ThemeError> {
        if let Some(parent) = export_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    ThemeError::ExportFailed(format!(
                        "Failed to create destination directory: {}",
                        e
                    ))
                })?;
            }
        }

        let file = File::create(export_path).map_err(|e| {
            ThemeError::ExportFailed(format!("Failed to create export file: {}", e))
        })?;

        let mut writer = ZipWriter::new(file);
        Self::add_directory_to_zip(&mut writer, bundle_root, bundle_root)?;
        writer
            .finish()
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to finalize archive: {}", e)))?;

        Ok(())
    }

    fn add_directory_to_zip<W: Write + Seek>(
        writer: &mut ZipWriter<W>,
        dir: &Path,
        base: &Path,
    ) -> Result<(), ThemeError> {
        for entry in fs::read_dir(dir).map_err(|e| {
            ThemeError::ExportFailed(format!("Failed to read bundle directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                ThemeError::ExportFailed(format!("Failed to read bundle entry: {}", e))
            })?;
            let path = entry.path();
            let rel_path = path.strip_prefix(base).map_err(|_| {
                ThemeError::ExportFailed("Failed to compute relative bundle path".to_string())
            })?;

            if entry
                .file_type()
                .map_err(|e| ThemeError::ExportFailed(format!("Failed to inspect entry: {}", e)))?
                .is_dir()
            {
                Self::add_directory_to_zip(writer, &path, base)?;
            } else {
                let rel_str = Self::normalize_path(rel_path);
                writer
                    .start_file(
                        rel_str,
                        FileOptions::<()>::default()
                            .compression_method(CompressionMethod::Deflated),
                    )
                    .map_err(|e| {
                        ThemeError::ExportFailed(format!("Failed to add file to archive: {}", e))
                    })?;

                let mut source = File::open(&path).map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to open bundle asset: {}", e))
                })?;
                std::io::copy(&mut source, writer).map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to write archive entry: {}", e))
                })?;
            }
        }

        Ok(())
    }

    fn collect_relative_files(root: &Path) -> Result<Vec<PathBuf>, ThemeError> {
        let mut files = Vec::new();
        Self::walk_bundle(root, root, &mut files)?;
        Ok(files)
    }

    fn walk_bundle(
        current: &Path,
        base: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), ThemeError> {
        for entry in fs::read_dir(current).map_err(|e| {
            ThemeError::ExportFailed(format!("Failed to read bundle directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                ThemeError::ExportFailed(format!("Failed to read bundle entry: {}", e))
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|e| {
                ThemeError::ExportFailed(format!("Failed to inspect bundle entry: {}", e))
            })?;

            if file_type.is_dir() {
                Self::walk_bundle(&path, base, files)?;
            } else {
                let rel_path = path.strip_prefix(base).map_err(|_| {
                    ThemeError::ExportFailed("Failed to compute relative bundle path".to_string())
                })?;
                files.push(rel_path.to_path_buf());
            }
        }

        Ok(())
    }

    fn copy_backgrounds_for_export(
        &self,
        theme_dir: &Path,
        bundle_root: &Path,
    ) -> Result<(), ThemeError> {
        let src = theme_dir.join("backgrounds");
        if !src.exists() {
            return Ok(());
        }

        let dest = bundle_root.join("backgrounds");
        fs::create_dir_all(&dest).map_err(|e| {
            ThemeError::ExportFailed(format!("Failed to prepare backgrounds: {}", e))
        })?;

        for entry in fs::read_dir(&src)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to read backgrounds: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                ThemeError::ExportFailed(format!("Failed to read background entry: {}", e))
            })?;
            let path = entry.path();
            if entry
                .file_type()
                .map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to inspect background: {}", e))
                })?
                .is_file()
            {
                let dest_path = dest.join(entry.file_name());
                fs::copy(&path, &dest_path).map_err(|e| {
                    ThemeError::ExportFailed(format!(
                        "Failed to copy background '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }

        Ok(())
    }

    fn copy_configs_for_export(
        &self,
        theme_dir: &Path,
        bundle_root: &Path,
        sanitized_name: &str,
    ) -> Result<(), ThemeError> {
        let entries = match fs::read_dir(theme_dir) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(ThemeError::ExportFailed(format!(
                    "Failed to read theme directory: {}",
                    e
                )))
            },
        };

        let metadata_files = [
            format!("{}.json", sanitized_name),
            "custom_theme.json".to_string(),
        ];

        let dest = bundle_root.join("configs");
        let mut copied_any = false;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ThemeError::ExportFailed(format!("Failed to read config entry: {}", e))
            })?;
            let path = entry.path();
            if !entry
                .file_type()
                .map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to inspect config entry: {}", e))
                })?
                .is_file()
            {
                continue;
            }

            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if metadata_files.iter().any(|m| m == &file_name_str) {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase());

            if !matches!(ext.as_deref(), Some("toml" | "conf" | "json" | "ini")) {
                continue;
            }

            if !dest.exists() {
                fs::create_dir_all(&dest).map_err(|e| {
                    ThemeError::ExportFailed(format!("Failed to create configs directory: {}", e))
                })?;
            }

            let dest_path = dest.join(&file_name);
            fs::copy(&path, &dest_path).map_err(|e| {
                ThemeError::ExportFailed(format!(
                    "Failed to copy config '{}': {}",
                    path.display(),
                    e
                ))
            })?;
            copied_any = true;
        }

        if !copied_any && dest.exists() {
            let _ = fs::remove_dir_all(dest);
        }

        Ok(())
    }

    fn copy_media_for_export(
        &self,
        theme_dir: &Path,
        bundle_root: &Path,
    ) -> Result<(), ThemeError> {
        let src = theme_dir.join("media");
        if !src.exists() {
            return Ok(());
        }

        Self::copy_directory_recursive(&src, &bundle_root.join("media"))
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to copy media assets: {}", e)))
    }

    fn copy_directory_recursive(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let target = dest.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                Self::copy_directory_recursive(&path, &target)?;
            } else {
                fs::copy(&path, &target)?;
            }
        }

        Ok(())
    }

    fn metadata_path(theme_dir: &Path, sanitized_name: &str) -> PathBuf {
        let new_path = theme_dir.join(format!("{}.json", sanitized_name));
        let old_path = theme_dir.join("custom_theme.json");

        if new_path.exists() {
            new_path
        } else if old_path.exists() {
            old_path
        } else {
            new_path
        }
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf, std::io::Error> {
        let path = std::env::temp_dir().join(format!("{}-{}", prefix, Uuid::new_v4()));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn compute_sha256_hex(path: &Path) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let read_bytes = file.read(&mut buffer)?;
            if read_bytes == 0 {
                break;
            }
            hasher.update(&buffer[..read_bytes]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn normalize_path(path: &Path) -> String {
        path.components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    }

    fn is_zip_archive(&self, file_path: &Path) -> Result<bool, ThemeError> {
        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(ThemeError::ImportFailed(format!(
                    "Failed to open file: {}",
                    e
                )))
            },
        };

        let mut signature = [0u8; 4];
        match file.read_exact(&mut signature) {
            Ok(_) => {},
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(false),
            Err(e) => {
                return Err(ThemeError::ImportFailed(format!(
                    "Failed to read file header: {}",
                    e
                )))
            },
        }

        Ok(matches!(
            &signature,
            b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08"
        ))
    }
}
