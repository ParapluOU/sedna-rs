use crate::error::{Result, SednaError};
use once_cell::sync::OnceCell;
use rust_embed::RustEmbed;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(RustEmbed)]
#[folder = "sedna_install/"]
struct SednaInstall;

static BINARIES_INSTANCE: OnceCell<Arc<ExtractedBinaries>> = OnceCell::new();

pub struct ExtractedBinaries {
    #[allow(dead_code)]
    temp_dir: TempDir,
    pub bin_dir: PathBuf,
}

impl ExtractedBinaries {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SednaError::BinaryExtractionFailed(format!("Failed to create temp dir: {}", e))
        })?;

        let bin_dir = temp_dir.path().join("bin");

        // Extract all embedded files using RustEmbed
        for file in SednaInstall::iter() {
            let file_path = file.as_ref();
            let embedded_file = SednaInstall::get(file_path).ok_or_else(|| {
                SednaError::BinaryExtractionFailed(format!("Failed to get embedded file: {}", file_path))
            })?;

            let dest_path = temp_dir.path().join(file_path);

            // Create parent directories if needed
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    SednaError::BinaryExtractionFailed(format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Write the file
            fs::write(&dest_path, embedded_file.data.as_ref()).map_err(|e| {
                SednaError::BinaryExtractionFailed(format!(
                    "Failed to write file {}: {}",
                    file_path, e
                ))
            })?;

            // Make binaries executable
            if file_path.starts_with("bin/") {
                let mut perms = fs::metadata(&dest_path)
                    .map_err(|e| {
                        SednaError::BinaryExtractionFailed(format!(
                            "Failed to get metadata for {}: {}",
                            file_path, e
                        ))
                    })?
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms).map_err(|e| {
                    SednaError::BinaryExtractionFailed(format!(
                        "Failed to set permissions for {}: {}",
                        file_path, e
                    ))
                })?;
            }
        }

        Ok(Self { temp_dir, bin_dir })
    }

    pub fn get_binary_path(&self, name: &str) -> PathBuf {
        self.bin_dir.join(name)
    }
}

pub fn get_binaries() -> Result<Arc<ExtractedBinaries>> {
    BINARIES_INSTANCE
        .get_or_try_init(|| ExtractedBinaries::new().map(Arc::new))
        .map(Arc::clone)
}
