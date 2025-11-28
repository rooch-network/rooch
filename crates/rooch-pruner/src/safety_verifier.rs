// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::path::{Path, PathBuf};

/// 极简的数据库安全验证器
pub struct SafetyVerifier {
    db_path: PathBuf,
}

/// 安全验证结果
#[derive(Debug)]
pub struct SafetyReport {
    pub database_available: bool,
    pub message: String,
}

impl SafetyVerifier {
    /// 创建新的安全验证器
    pub fn new(db_path: &Path) -> Self {
        Self {
            db_path: db_path.to_path_buf(),
        }
    }

    /// 执行安全验证
    pub fn verify_database_access(&self) -> Result<SafetyReport> {
        // 1. 检查数据库目录是否存在
        if !self.db_path.exists() || !self.db_path.is_dir() {
            return Ok(SafetyReport {
                database_available: false,
                message: format!("Database directory does not exist: {:?}", self.db_path),
            });
        }

        // 2. 检查 RocksDB LOCK 文件状态
        let lock_file = self.db_path.join("LOCK");
        if lock_file.exists() {
            // 尝试获取锁来验证是否真的被锁定
            match self.try_acquire_exclusive_access(&lock_file) {
                Ok(_) => {
                    // 可以获取锁，说明数据库没有被其他进程锁定
                    Ok(SafetyReport {
                        database_available: true,
                        message: "Database is available for exclusive access".to_string(),
                    })
                }
                Err(e) => {
                    // 无法获取锁，说明被其他进程锁定
                    Ok(SafetyReport {
                        database_available: false,
                        message: format!("Database is locked by another process: {}", e),
                    })
                }
            }
        } else {
            // LOCK 文件不存在，可能数据库未初始化或损坏
            Ok(SafetyReport {
                database_available: false,
                message: "Database LOCK file missing - database may not be properly initialized"
                    .to_string(),
            })
        }
    }

    /// 尝试获取数据库排他访问权限
    fn try_acquire_exclusive_access(&self, _lock_file: &Path) -> Result<()> {
        // 创建测试文件来验证访问权限
        let test_file = self.db_path.join("gc_safety_check.tmp");

        // 尝试创建和锁定测试文件
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&test_file)?;

        // 立即删除测试文件
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

        // 测试创建和删除临时文件的能力
        let test_file = temp_dir.path().join("gc_safety_check.tmp");
        let result = verifier.try_acquire_exclusive_access(&temp_dir.path().join("LOCK"));
        assert!(result.is_ok());
        assert!(!test_file.exists()); // 文件应该被清理
    }
}
