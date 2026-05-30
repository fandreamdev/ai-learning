//! 密码工具模块
//!
//! 提供密码哈希和验证功能

use crate::config::SecurityConfig;
use crate::error::AppResult;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// 密码工具
#[derive(Clone)]
pub struct PasswordUtils {
    argon2: Argon2<'static>,
}

impl PasswordUtils {
    /// 创建新的密码工具实例
    pub fn new(config: &SecurityConfig) -> Self {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                config.argon2.memory_cost,
                config.argon2.time_cost,
                config.argon2.parallelism,
                None,
            )
            .expect("Invalid Argon2 parameters"),
        );

        Self { argon2 }
    }

    /// 哈希密码
    pub fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| crate::error::AppError::internal(format!("Password hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }

    /// 验证密码
    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// 检查密码强度
    pub fn check_password_strength(password: &str) -> PasswordStrength {
        let len = password.len();
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let score = [has_lower, has_upper, has_digit, has_special]
            .iter()
            .filter(|&&b| b)
            .count();

        if len < 8 {
            PasswordStrength::TooShort
        } else if score < 2 {
            PasswordStrength::Weak
        } else if score < 3 || len < 12 {
            PasswordStrength::Medium
        } else {
            PasswordStrength::Strong
        }
    }
}

/// 密码强度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// 太短
    TooShort,
    /// 弱
    Weak,
    /// 中等
    Medium,
    /// 强
    Strong,
}

impl PasswordStrength {
    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::TooShort => "密码长度至少为 8 个字符",
            Self::Weak => "密码强度弱，建议包含大小写字母、数字和特殊字符",
            Self::Medium => "密码强度中等",
            Self::Strong => "密码强度强",
        }
    }

    /// 是否符合最低要求
    pub fn meets_minimum(&self) -> bool {
        !matches!(self, Self::TooShort | Self::Weak)
    }
}

impl Default for PasswordUtils {
    fn default() -> Self {
        use crate::config::{Argon2Config, SecurityConfig};

        Self::new(&SecurityConfig {
            argon2: Argon2Config {
                memory_cost: 19456,
                time_cost: 2,
                parallelism: 1,
            },
            cors: Default::default(),
            sql: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_password_utils() -> PasswordUtils {
        PasswordUtils::default()
    }

    #[test]
    fn test_hash_and_verify() {
        let utils = create_test_password_utils();
        let password = "TestPassword123!";

        let hash = utils.hash_password(password).unwrap();
        assert_ne!(hash, password);

        assert!(utils.verify_password(password, &hash));
        assert!(!utils.verify_password("WrongPassword", &hash));
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let utils = create_test_password_utils();
        let password = "TestPassword123!";

        let hash1 = utils.hash_password(password).unwrap();
        let hash2 = utils.hash_password(password).unwrap();

        // 相同的密码应该生成不同的哈希（因为 salt 不同）
        assert_ne!(hash1, hash2);

        // 但两个哈希都应该验证通过
        assert!(utils.verify_password(password, &hash1));
        assert!(utils.verify_password(password, &hash2));
    }

    #[test]
    fn test_invalid_hash() {
        let utils = create_test_password_utils();

        // 无效的哈希格式应该返回 false
        assert!(!utils.verify_password("password", "invalid-hash"));
        assert!(!utils.verify_password("password", ""));
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(
            PasswordUtils::check_password_strength("abc"),
            PasswordStrength::TooShort
        );

        assert_eq!(
            PasswordUtils::check_password_strength("password"),
            PasswordStrength::Weak
        );

        assert_eq!(
            PasswordUtils::check_password_strength("Password1"),
            PasswordStrength::Weak
        );

        assert_eq!(
            PasswordUtils::check_password_strength("Password123"),
            PasswordStrength::Medium
        );

        assert_eq!(
            PasswordUtils::check_password_strength("Password123!"),
            PasswordStrength::Strong
        );

        assert_eq!(
            PasswordUtils::check_password_strength("VeryLongPassword123!@#"),
            PasswordStrength::Strong
        );
    }

    #[test]
    fn test_password_strength_minimum() {
        assert!(!PasswordUtils::check_password_strength("abc").meets_minimum());
        assert!(!PasswordUtils::check_password_strength("password").meets_minimum());
        assert!(PasswordUtils::check_password_strength("Password123").meets_minimum());
        assert!(PasswordUtils::check_password_strength("StrongPass!").meets_minimum());
    }
}
