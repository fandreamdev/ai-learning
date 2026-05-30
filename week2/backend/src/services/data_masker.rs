//! 数据脱敏服务
//!
//! 提供敏感数据检测和脱敏功能

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 脱敏规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRule {
    /// 字段名模式（正则或精确匹配）
    pub field_pattern: String,

    /// 字段类型
    pub field_type: FieldType,

    /// 是否启用
    pub enabled: bool,
}

/// 字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// 手机号
    Phone,
    /// 身份证
    IdCard,
    /// 邮箱
    Email,
    /// 银行卡
    BankCard,
    /// 姓名
    Name,
    /// 地址
    Address,
    /// 薪资
    Salary,
    /// 密码
    Password,
    /// 自定义
    Custom,
}

impl FieldType {
    pub fn from_field_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();

        if lower.contains("phone") || lower.contains("mobile") || lower.contains("tel") {
            Some(Self::Phone)
        } else if lower.contains("idcard") || lower.contains("id_card") || lower.contains("身份证") {
            Some(Self::IdCard)
        } else if lower.contains("email") || lower.contains("mail") {
            Some(Self::Email)
        } else if lower.contains("bank") || lower.contains("card") || lower.contains("银行卡") {
            Some(Self::BankCard)
        } else if lower.contains("name") || lower.contains("姓名") || lower.contains("username") {
            Some(Self::Name)
        } else if lower.contains("address") || lower.contains("addr") || lower.contains("地址") {
            Some(Self::Address)
        } else if lower.contains("salary") || lower.contains("wage") || lower.contains("薪资") || lower.contains("工资") {
            Some(Self::Salary)
        } else if lower.contains("password") || lower.contains("pwd") || lower.contains("密码") {
            Some(Self::Password)
        } else {
            None
        }
    }
}

/// 数据脱敏器
#[derive(Clone)]
pub struct DataMasker {
    rules: Vec<MaskingRule>,
}

impl DataMasker {
    pub fn new() -> Self {
        let default_rules = vec![
            MaskingRule {
                field_pattern: "phone|mobile|tel".to_string(),
                field_type: FieldType::Phone,
                enabled: true,
            },
            MaskingRule {
                field_pattern: "idcard|id_card|身份证".to_string(),
                field_type: FieldType::IdCard,
                enabled: true,
            },
            MaskingRule {
                field_pattern: "email|mail".to_string(),
                field_type: FieldType::Email,
                enabled: true,
            },
            MaskingRule {
                field_pattern: "bank|card|银行卡".to_string(),
                field_type: FieldType::BankCard,
                enabled: true,
            },
            MaskingRule {
                field_pattern: "name|姓名|username".to_string(),
                field_type: FieldType::Name,
                enabled: true,
            },
            MaskingRule {
                field_pattern: "salary|wage|薪资|工资".to_string(),
                field_type: FieldType::Salary,
                enabled: true,
            },
            MaskingRule {
                field_pattern: "password|pwd|密码".to_string(),
                field_type: FieldType::Password,
                enabled: true,
            },
        ];

        Self { rules: default_rules }
    }

    /// 添加自定义规则
    pub fn add_rule(&mut self, rule: MaskingRule) {
        self.rules.push(rule);
    }

    /// 检测字段类型
    pub fn detect_field_type(&self, field_name: &str) -> Option<FieldType> {
        // 先尝试规则匹配
        for rule in &self.rules {
            if rule.enabled {
                let re = Regex::new(&rule.field_pattern).ok()?;
                if re.is_match(&field_name.to_lowercase()) {
                    return Some(rule.field_type);
                }
            }
        }

        // 使用内置检测
        FieldType::from_field_name(field_name)
    }

    /// 脱敏数据
    pub fn mask_value(&self, value: &str, field_type: &FieldType) -> String {
        match field_type {
            FieldType::Phone => self.mask_phone(value),
            FieldType::IdCard => self.mask_id_card(value),
            FieldType::Email => self.mask_email(value),
            FieldType::BankCard => self.mask_bank_card(value),
            FieldType::Name => self.mask_name(value),
            FieldType::Address => self.mask_address(value),
            FieldType::Salary => self.mask_salary(value),
            FieldType::Password => "******".to_string(),
            FieldType::Custom => value.to_string(),
        }
    }

    /// 脱敏手机号（中间4位）
    fn mask_phone(&self, value: &str) -> String {
        // 移除非数字字符
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() == 11 {
            format!("{}****{}", &digits[..3], &digits[7..])
        } else if digits.len() >= 7 {
            format!("{}****{}", &digits[..3], &digits[digits.len() - 4..])
        } else {
            "***".to_string()
        }
    }

    /// 脱敏身份证（前3后4）
    fn mask_id_card(&self, value: &str) -> String {
        if value.len() >= 15 {
            let len = value.len();
            format!("{}**********{}", &value[..3], &value[len - 4..])
        } else {
            "**************".to_string()
        }
    }

    /// 脱敏邮箱（域名前缀掩码）
    fn mask_email(&self, value: &str) -> String {
        if let Some(at_pos) = value.find('@') {
            let local = &value[..at_pos];
            let domain = &value[at_pos..];

            if local.len() <= 1 {
                format!("*{}", domain)
            } else if local.len() == 2 {
                format!("{}*{}", &local[..1], domain)
            } else {
                format!("{}***{}", &local[..1], domain)
            }
        } else {
            value.to_string()
        }
    }

    /// 脱敏银行卡（前6后4）
    fn mask_bank_card(&self, value: &str) -> String {
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() >= 10 {
            format!("{}******{}", &digits[..6], &digits[digits.len() - 4..])
        } else {
            "****".to_string()
        }
    }

    /// 脱敏姓名（保留首尾字符）
    fn mask_name(&self, value: &str) -> String {
        let chars: Vec<char> = value.chars().collect();

        if chars.len() <= 1 {
            "*".to_string()
        } else if chars.len() == 2 {
            format!("{}*", &value[..1])
        } else {
            let mut result = value[..1].to_string();
            result.push_str(&"*".repeat(chars.len() - 2));
            result.push(chars.last().unwrap());
            result
        }
    }

    /// 脱敏地址（只保留省市区）
    fn mask_address(&self, value: &str) -> String {
        // 简化处理：只显示前6个字符
        if value.len() <= 6 {
            value.to_string()
        } else {
            format!("{}...", &value[..6])
        }
    }

    /// 脱敏薪资（范围模糊）
    fn mask_salary(&self, value: &str) -> String {
        // 解析数值
        let num: f64 = value.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect::<String>().parse().unwrap_or(0.0);

        if num == 0.0 {
            return "面议".to_string();
        }

        let (min, max) = if num < 5000.0 {
            ("3k", "5k")
        } else if num < 10000.0 {
            ("5k", "10k")
        } else if num < 20000.0 {
            ("10k", "20k")
        } else if num < 30000.0 {
            ("20k", "30k")
        } else {
            ("30k", "50k")
        };

        format!("{}-{}", min, max)
    }

    /// 批量脱敏数据行
    pub fn mask_row(&self, row: &HashMap<String, serde_json::Value>) -> HashMap<String, serde_json::Value> {
        let mut masked = row.clone();

        for (field, value) in row.iter() {
            if let Some(field_type) = self.detect_field_type(field) {
                if let Some(str_value) = value.as_str() {
                    let masked_value = self.mask_value(str_value, &field_type);
                    masked.insert(field.clone(), serde_json::Value::String(masked_value));
                }
            }
        }

        masked
    }
}

impl Default for DataMasker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_masker() -> DataMasker {
        DataMasker::new()
    }

    #[test]
    fn test_mask_phone() {
        let masker = create_test_masker();

        assert_eq!(masker.mask_phone("13812345678"), "138****5678");
        assert_eq!(masker.mask_phone("138-1234-5678"), "138****5678");
        assert_eq!(masker.mask_phone("12345"), "***");
    }

    #[test]
    fn test_mask_id_card() {
        let masker = create_test_masker();

        assert_eq!(masker.mask_id_card("110101199001011234"), "110**********1234");
        assert_eq!(masker.mask_id_card("123456789012345"), "123**********2345");
    }

    #[test]
    fn test_mask_email() {
        let masker = create_test_masker();

        assert_eq!(masker.mask_email("test@example.com"), "t***@example.com");
        assert_eq!(masker.mask_email("ab@example.com"), "a*@example.com");
        assert_eq!(masker.mask_email("@example.com"), "*@example.com");
    }

    #[test]
    fn test_mask_bank_card() {
        let masker = create_test_masker();

        assert_eq!(masker.mask_bank_card("6222021234567890123"), "622202******0123");
    }

    #[test]
    fn test_mask_name() {
        let masker = create_test_masker();

        assert_eq!(masker.mask_name("张三"), "张*");
        assert_eq!(masker.mask_name("李四"), "李*");
        assert_eq!(masker.mask_name("王五五"), "王**");
        assert_eq!(masker.mask_name("赵六六六"), "赵****六");
    }

    #[test]
    fn test_mask_salary() {
        let masker = create_test_masker();

        assert_eq!(masker.mask_salary("5000"), "3k-5k");
        assert_eq!(masker.mask_salary("8000"), "5k-10k");
        assert_eq!(masker.mask_salary("15000"), "10k-20k");
        assert_eq!(masker.mask_salary("25000"), "20k-30k");
        assert_eq!(masker.mask_salary("40000"), "30k-50k");
        assert_eq!(masker.mask_salary("abc"), "面议");
    }

    #[test]
    fn test_detect_field_type() {
        let masker = create_test_masker();

        assert_eq!(masker.detect_field_type("phone_number"), Some(FieldType::Phone));
        assert_eq!(masker.detect_field_type("mobile"), Some(FieldType::Phone));
        assert_eq!(masker.detect_field_type("user_email"), Some(FieldType::Email));
        assert_eq!(masker.detect_field_type("bank_card_no"), Some(FieldType::BankCard));
        assert_eq!(masker.detect_field_type("user_name"), Some(FieldType::Name));
        assert_eq!(masker.detect_field_type("salary"), Some(FieldType::Salary));
        assert_eq!(masker.detect_field_type("password"), Some(FieldType::Password));
        assert_eq!(masker.detect_field_type("id_card"), Some(FieldType::IdCard));
        assert_eq!(masker.detect_field_type("unknown_field"), None);
    }

    #[test]
    fn test_mask_row() {
        let masker = create_test_masker();

        let mut row = HashMap::new();
        row.insert("name".to_string(), serde_json::json!("张三"));
        row.insert("phone".to_string(), serde_json::json!("13812345678"));
        row.insert("email".to_string(), serde_json::json!("test@example.com"));
        row.insert("age".to_string(), serde_json::json!(25));

        let masked = masker.mask_row(&row);

        assert_eq!(masked.get("name").and_then(|v| v.as_str()), Some("张*"));
        assert_eq!(masked.get("phone").and_then(|v| v.as_str()), Some("138****5678"));
        assert_eq!(masked.get("email").and_then(|v| v.as_str()), Some("t***@example.com"));
        assert_eq!(masked.get("age").and_then(|v| v.as_i64()), Some(25)); // 未检测到类型，保持原值
    }
}
