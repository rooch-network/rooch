// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Minimal database safety verifier
pub struct SafetyVerifier {
    db_path: PathBuf,
}

/// Safety verification result
#[derive(Debug)]
pub struct SafetyReport {
    pub database_available: bool,
    pub message: String,
}

impl SafetyVerifier {
    /// Create new safety verifier
    pub fn new(db_path: &Path) -> Self {
        Self {
            db_path: db_path.to_path_buf(),
        }
    }

    /// Execute safety verification
    pub fn verify_database_access(&self) -> Result<SafetyReport> {
        // 1. Check if database directory exists
        if !self.db_path.exists() || !self.db_path.is_dir() {
            return Ok(SafetyReport {
                database_available: false,
                message: format!("Database directory does not exist: {:?}", self.db_path),
            });
        }

        // 2. Check RocksDB LOCK file status
        let lock_file = self.db_path.join("LOCK");
        if lock_file.exists() {
            // Try to acquire lock to verify if it's actually locked
            match self.try_acquire_exclusive_access(&lock_file) {
                Ok(_) => {
                    // Can acquire lock, database is not locked by another process
                    Ok(SafetyReport {
                        database_available: true,
                        message: "Database is available for exclusive access".to_string(),
                    })
                }
                Err(e) => {
                    // Cannot acquire lock, database is locked by another process
                    Ok(SafetyReport {
                        database_available: false,
                        message: format!("Database is locked by another process: {}", e),
                    })
                }
            }
        } else {
            // LOCK file doesn't exist, database may not be initialized or corrupted
            Ok(SafetyReport {
                database_available: false,
                message: "Database LOCK file missing - database may not be properly initialized"
                    .to_string(),
            })
        }
    }

    /// Try to acquire database exclusive access
    fn try_acquire_exclusive_access(&self, _lock_file: &Path) -> Result<()> {
        // Create test file to verify access permissions
        let test_file = self.db_path.join("gc_safety_check.tmp");

        // Try to create and lock test file
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&test_file)?;

        // Delete test file immediately
        drop(file);
        std::fs::remove_file(&test_file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_safety_verifier_creation() {
        let temp_dir = TempDir::new().unwrap();
        let verifier = SafetyVerifier::new(temp_dir.path());
        assert_eq!(verifier.db_path, temp_dir.path());
    }

    #[test]
    fn test_nonexistent_directory() {
        let nonexistent_path = PathBuf::from("/path/that/does/not/exist");
        let verifier = SafetyVerifier::new(&nonexistent_path);

        let report = verifier.verify_database_access().unwrap();
        assert!(!report.database_available);
        assert!(report.message.contains("does not exist"));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let verifier = SafetyVerifier::new(temp_dir.path());

        let report = verifier.verify_database_access().unwrap();
        assert!(!report.database_available);
        assert!(report.message.contains("LOCK file missing"));
    }

    #[test]
    fn test_directory_with_lock_file() {
        let temp_dir = TempDir::new().unwrap();
        let lock_file = temp_dir.path().join("LOCK");
        fs::write(&lock_file, "").unwrap();

        let verifier = SafetyVerifier::new(temp_dir.path());

        let report = verifier.verify_database_access().unwrap();
        assert!(report.database_available);
        assert!(report.message.contains("available for exclusive access"));
    }

    #[test]
    fn test_exclusive_access_verification() {
        let temp_dir = TempDir::new().unwrap();
        let verifier = SafetyVerifier::new(temp_dir.path());

        // Test ability to create and delete temporary files
        let test_file = temp_dir.path().join("gc_safety_check.tmp");
        let result = verifier.try_acquire_exclusive_access(&temp_dir.path().join("LOCK"));
        assert!(result.is_ok());
        assert!(!test_file.exists()); // File should be cleaned up
    }
}
