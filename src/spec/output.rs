//! Spec file output module for SpecGen CLI.
//!
//! This module handles atomic file writing with existence checks.

use crate::error::SpecGenError;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use std::io::Write;

/// Output directory for spec files.
const SPEC_OUTPUT_DIR: &str = "specs";

/// Write content to a file atomically.
///
/// Uses a temporary file + rename pattern to ensure the file is either
/// fully written or not present at all (no partial writes).
///
/// # Arguments
/// * `path` - The target file path (relative or absolute)
/// * `content` - The content to write
///
/// # Errors
/// Returns an error if:
/// - The file already exists and allow_overwrite is false
/// - The parent directory cannot be created
/// - The file cannot be written
pub fn write_spec_file(
    path: &Utf8Path,
    content: &str,
    allow_overwrite: bool,
) -> Result<(), SpecGenError> {
    // Check for path traversal attempts
    validate_path(path)?;

    // Check if file exists (unless overwriting)
    if !allow_overwrite && path.exists() {
        return Err(SpecGenError::FileExists(path.to_string()));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to temporary file first
    let _temp_path = path.with_extension("tmp");
    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    // Atomically rename temp file to target
    file.persist(path)
        .map_err(|e| SpecGenError::IoError(format!("Failed to persist temp file: {}", e)))?;

    Ok(())
}

/// Write multiple spec sections to files.
///
/// This is the main entry point for writing all generated spec sections.
/// It checks that no files exist before writing (unless overwrite is allowed).
///
/// # Arguments
/// * `sections` - Vector of (section_name, content) tuples
/// * `output_dir` - The output directory path
/// * `allow_overwrite` - Whether to allow overwriting existing files
pub fn write_all_sections(
    sections: &[(&str, &str)],
    output_dir: &Utf8Path,
    allow_overwrite: bool,
) -> Result<Vec<Utf8PathBuf>, SpecGenError> {
    let mut written_paths = Vec::new();

    // First, check all files don't exist (unless overwriting)
    if !allow_overwrite {
        for (section_name, _) in sections {
            let file_path = output_dir.join(format!("{}.md", section_name));
            if file_path.exists() {
                return Err(SpecGenError::FileExists(format!(
                    "Spec file '{}' already exists. Use --diff to see changes or --force to overwrite.",
                    file_path
                )));
            }
        }
    }

    // Write all files
    for (section_name, content) in sections {
        let file_path = output_dir.join(format!("{}.md", section_name));
        write_spec_file(&file_path, content, allow_overwrite)?;
        written_paths.push(file_path);
    }

    Ok(written_paths)
}

/// Validate that a path is safe to write to.
///
/// Prevents path traversal attacks by checking for:
/// - Parent directory references (..)
fn validate_path(path: &Utf8Path) -> Result<(), SpecGenError> {
    // Check for path traversal attempts
    let path_str = path.as_str();
    if path_str.contains("..") {
        return Err(SpecGenError::InvalidPath(
            "Path traversal detected".to_string(),
        ));
    }

    Ok(())
}

/// Get the default output directory for specs.
pub fn get_default_output_dir() -> Utf8PathBuf {
    Utf8PathBuf::from(SPEC_OUTPUT_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_spec_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
            .unwrap()
            .join("test.md");

        let result = write_spec_file(&temp_path, "# Test Content", false);
        assert!(result.is_ok());
        assert!(temp_path.exists());
    }

    #[test]
    fn test_write_spec_file_no_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
            .unwrap()
            .join("test.md");

        // First write should succeed
        write_spec_file(&temp_path, "# Test Content", false).unwrap();

        // Second write without overwrite should fail
        let result = write_spec_file(&temp_path, "# New Content", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_spec_file_with_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
            .unwrap()
            .join("test.md");

        // First write
        write_spec_file(&temp_path, "# Test Content", false).unwrap();

        // Second write with overwrite should succeed
        let result = write_spec_file(&temp_path, "# New Content", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        let result = validate_path(Utf8Path::new("../etc/passwd"));
        assert!(result.is_err());

        let result = validate_path(Utf8Path::new("foo/../../../bar"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safe_path() {
        let result = validate_path(Utf8Path::new("specs/requirements.md"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_all_sections() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();

        let sections = vec![
            ("requirements", "# Requirements\n\nFR-001: Test requirement"),
            ("architecture", "# Architecture\n\nTest architecture"),
        ];

        let result = write_all_sections(&sections, &output_dir, false);
        assert!(result.is_ok());

        let paths = result.unwrap();
        assert_eq!(paths.len(), 2);
        assert!(output_dir.join("requirements.md").exists());
        assert!(output_dir.join("architecture.md").exists());
    }
}
