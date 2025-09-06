use anyhow::{anyhow, Result};
use md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// MD5 hash function
pub struct Md5Function;

impl SqlFunction for Md5Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MD5",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate MD5 hash of a string",
            returns: "String (32 character hex digest)",
            examples: vec![
                "MD5('hello') = '5d41402abc4b2a76b9719d911017c592'",
                "MD5('test') = '098f6bcd4621d373cade4e832627b4f6'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let input_string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(idx) => idx.to_string(), // Convert interned string index to string
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
        };

        let digest = md5::compute(input_string.as_bytes());
        Ok(DataValue::String(format!("{:x}", digest)))
    }
}

/// SHA1 hash function
pub struct Sha1Function;

impl SqlFunction for Sha1Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SHA1",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate SHA1 hash of a string",
            returns: "String (40 character hex digest)",
            examples: vec![
                "SHA1('hello') = '2aae6c35c94fcfb415dbe95f408b9ce91ee846ed'",
                "SHA1('test') = 'a94a8fe5ccb19ba61c4c0873d391e987982fbbd3'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let input_string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(idx) => idx.to_string(), // Convert interned string index to string
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
        };

        let mut hasher = Sha1::new();
        hasher.update(input_string.as_bytes());
        let result = hasher.finalize();
        Ok(DataValue::String(format!("{:x}", result)))
    }
}

/// SHA256 hash function
pub struct Sha256Function;

impl SqlFunction for Sha256Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SHA256",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate SHA256 hash of a string",
            returns: "String (64 character hex digest)",
            examples: vec![
                "SHA256('hello') = '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'",
                "SHA256('test') = '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let input_string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(idx) => idx.to_string(), // Convert interned string index to string
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
        };

        let mut hasher = Sha256::new();
        hasher.update(input_string.as_bytes());
        let result = hasher.finalize();
        Ok(DataValue::String(format!("{:x}", result)))
    }
}

/// SHA512 hash function
pub struct Sha512Function;

impl SqlFunction for Sha512Function {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SHA512",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate SHA512 hash of a string",
            returns: "String (128 character hex digest)",
            examples: vec![
                "SHA512('hello') = '9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043'",
                "SHA512('test') = 'ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let input_string = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(idx) => idx.to_string(), // Convert interned string index to string
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::DateTime(dt) => dt.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
        };

        let mut hasher = Sha512::new();
        hasher.update(input_string.as_bytes());
        let result = hasher.finalize();
        Ok(DataValue::String(format!("{:x}", result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let func = Md5Function;

        // Test basic string
        let result = func
            .evaluate(&[DataValue::String("hello".to_string())])
            .unwrap();
        assert_eq!(
            result,
            DataValue::String("5d41402abc4b2a76b9719d911017c592".to_string())
        );

        // Test NULL
        let result = func.evaluate(&[DataValue::Null]).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_sha1() {
        let func = Sha1Function;

        // Test basic string
        let result = func
            .evaluate(&[DataValue::String("test".to_string())])
            .unwrap();
        assert_eq!(
            result,
            DataValue::String("a94a8fe5ccb19ba61c4c0873d391e987982fbbd3".to_string())
        );

        // Test NULL
        let result = func.evaluate(&[DataValue::Null]).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_sha256() {
        let func = Sha256Function;

        // Test basic string
        let result = func
            .evaluate(&[DataValue::String("test".to_string())])
            .unwrap();
        assert_eq!(
            result,
            DataValue::String(
                "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_string()
            )
        );

        // Test NULL
        let result = func.evaluate(&[DataValue::Null]).unwrap();
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_sha512() {
        let func = Sha512Function;

        // Test basic string
        let result = func
            .evaluate(&[DataValue::String("test".to_string())])
            .unwrap();
        assert_eq!(result, DataValue::String("ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff".to_string()));

        // Test empty string
        let result = func.evaluate(&[DataValue::String("".to_string())]).unwrap();
        assert_eq!(result, DataValue::String("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string()));
    }
}
