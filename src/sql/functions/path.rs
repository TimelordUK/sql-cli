use std::path::Path;

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

fn as_str(v: &DataValue) -> Option<String> {
    match v {
        DataValue::String(s) => Some(s.clone()),
        DataValue::InternedString(s) => Some(s.as_ref().to_string()),
        DataValue::Null => None,
        _ => Some(v.to_string()),
    }
}

/// BASENAME - last component of a path (file name with extension)
pub struct BasenameFunction;

impl SqlFunction for BasenameFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "BASENAME",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Return the last component of a path (file name with extension)",
            returns: "STRING (or NULL if path has no file name)",
            examples: vec![
                "SELECT BASENAME('/home/me/src/main.rs')  -- 'main.rs'",
                "SELECT BASENAME(path) FROM files",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let Some(s) = as_str(&args[0]) else {
            return Ok(DataValue::Null);
        };
        match Path::new(&s).file_name() {
            Some(name) => Ok(DataValue::String(name.to_string_lossy().into_owned())),
            None => Ok(DataValue::Null),
        }
    }
}

/// DIRNAME - everything except the last component
pub struct DirnameFunction;

impl SqlFunction for DirnameFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DIRNAME",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Return the parent directory of a path",
            returns: "STRING (or NULL if path has no parent)",
            examples: vec![
                "SELECT DIRNAME('/home/me/src/main.rs')  -- '/home/me/src'",
                "SELECT DIRNAME(path) FROM files",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let Some(s) = as_str(&args[0]) else {
            return Ok(DataValue::Null);
        };
        match Path::new(&s).parent() {
            Some(parent) => {
                let out = parent.to_string_lossy();
                if out.is_empty() {
                    Ok(DataValue::Null)
                } else {
                    Ok(DataValue::String(out.into_owned()))
                }
            }
            None => Ok(DataValue::Null),
        }
    }
}

/// EXTENSION - file extension (without leading dot)
pub struct ExtensionFunction;

impl SqlFunction for ExtensionFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "EXTENSION",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Return the file extension (without leading dot), or NULL if none",
            returns: "STRING or NULL",
            examples: vec![
                "SELECT EXTENSION('/home/me/src/main.rs')  -- 'rs'",
                "SELECT EXTENSION('README')                 -- NULL",
                "SELECT EXTENSION(path) FROM files",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let Some(s) = as_str(&args[0]) else {
            return Ok(DataValue::Null);
        };
        match Path::new(&s).extension() {
            Some(ext) => Ok(DataValue::String(ext.to_string_lossy().into_owned())),
            None => Ok(DataValue::Null),
        }
    }
}

/// STEM - file name without extension
pub struct StemFunction;

impl SqlFunction for StemFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "STEM",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Return the file name without its extension",
            returns: "STRING or NULL",
            examples: vec![
                "SELECT STEM('/home/me/src/main.rs')  -- 'main'",
                "SELECT STEM('archive.tar.gz')         -- 'archive.tar'",
                "SELECT STEM(path) FROM files",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let Some(s) = as_str(&args[0]) else {
            return Ok(DataValue::Null);
        };
        match Path::new(&s).file_stem() {
            Some(stem) => Ok(DataValue::String(stem.to_string_lossy().into_owned())),
            None => Ok(DataValue::Null),
        }
    }
}

/// PATH_DEPTH - number of components in the path
pub struct PathDepthFunction;

impl SqlFunction for PathDepthFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PATH_DEPTH",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(1),
            description: "Return the number of components in a path",
            returns: "INTEGER",
            examples: vec![
                "SELECT PATH_DEPTH('/home/me/src/main.rs')  -- 5",
                "SELECT PATH_DEPTH('src/main.rs')            -- 2",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let Some(s) = as_str(&args[0]) else {
            return Ok(DataValue::Null);
        };
        let count = Path::new(&s).components().count() as i64;
        Ok(DataValue::Integer(count))
    }
}

/// PATH_PART - nth component (1-based; negative = from end, -1 = last)
pub struct PathPartFunction;

impl SqlFunction for PathPartFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PATH_PART",
            category: FunctionCategory::String,
            arg_count: ArgCount::Fixed(2),
            description: "Return the nth component of a path (1-based; negative counts from the end, -1 = last)",
            returns: "STRING or NULL if index out of range",
            examples: vec![
                "SELECT PATH_PART('/home/me/src/main.rs', 1)   -- 'home'",
                "SELECT PATH_PART('/home/me/src/main.rs', -1)  -- 'main.rs'",
                "SELECT PATH_PART('/home/me/src/main.rs', -2)  -- 'src'",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let Some(s) = as_str(&args[0]) else {
            return Ok(DataValue::Null);
        };
        let n = match &args[1] {
            DataValue::Integer(n) => *n,
            DataValue::Float(f) => *f as i64,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("PATH_PART index must be an integer")),
        };

        let parts: Vec<String> = Path::new(&s)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();

        if parts.is_empty() || n == 0 {
            return Ok(DataValue::Null);
        }

        let idx = if n > 0 {
            (n - 1) as usize
        } else {
            let from_end = (-n) as usize;
            if from_end > parts.len() {
                return Ok(DataValue::Null);
            }
            parts.len() - from_end
        };

        match parts.get(idx) {
            Some(part) => Ok(DataValue::String(part.clone())),
            None => Ok(DataValue::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> DataValue {
        DataValue::String(v.to_string())
    }

    #[test]
    fn basename_extracts_file_name() {
        let f = BasenameFunction;
        assert_eq!(
            f.evaluate(&[s("/home/me/src/main.rs")]).unwrap(),
            s("main.rs")
        );
        assert_eq!(f.evaluate(&[s("main.rs")]).unwrap(), s("main.rs"));
        assert_eq!(f.evaluate(&[DataValue::Null]).unwrap(), DataValue::Null);
    }

    #[test]
    fn dirname_extracts_parent() {
        let f = DirnameFunction;
        assert_eq!(
            f.evaluate(&[s("/home/me/src/main.rs")]).unwrap(),
            s("/home/me/src")
        );
        assert_eq!(f.evaluate(&[s("main.rs")]).unwrap(), DataValue::Null);
    }

    #[test]
    fn extension_returns_ext_or_null() {
        let f = ExtensionFunction;
        assert_eq!(f.evaluate(&[s("main.rs")]).unwrap(), s("rs"));
        assert_eq!(f.evaluate(&[s("archive.tar.gz")]).unwrap(), s("gz"));
        assert_eq!(f.evaluate(&[s("README")]).unwrap(), DataValue::Null);
        assert_eq!(f.evaluate(&[s(".gitignore")]).unwrap(), DataValue::Null);
    }

    #[test]
    fn stem_strips_last_extension() {
        let f = StemFunction;
        assert_eq!(f.evaluate(&[s("main.rs")]).unwrap(), s("main"));
        assert_eq!(f.evaluate(&[s("archive.tar.gz")]).unwrap(), s("archive.tar"));
        assert_eq!(f.evaluate(&[s("README")]).unwrap(), s("README"));
    }

    #[test]
    fn path_depth_counts_components() {
        let f = PathDepthFunction;
        // "/home/me/src/main.rs" => [/, home, me, src, main.rs] = 5
        assert_eq!(
            f.evaluate(&[s("/home/me/src/main.rs")]).unwrap(),
            DataValue::Integer(5)
        );
        assert_eq!(
            f.evaluate(&[s("src/main.rs")]).unwrap(),
            DataValue::Integer(2)
        );
    }

    #[test]
    fn path_part_handles_positive_and_negative() {
        let f = PathPartFunction;
        let p = s("/home/me/src/main.rs");
        assert_eq!(f.evaluate(&[p.clone(), DataValue::Integer(-1)]).unwrap(), s("main.rs"));
        assert_eq!(f.evaluate(&[p.clone(), DataValue::Integer(-2)]).unwrap(), s("src"));
        assert_eq!(f.evaluate(&[p.clone(), DataValue::Integer(2)]).unwrap(), s("home"));
        assert_eq!(
            f.evaluate(&[p.clone(), DataValue::Integer(99)]).unwrap(),
            DataValue::Null
        );
        assert_eq!(
            f.evaluate(&[p, DataValue::Integer(0)]).unwrap(),
            DataValue::Null
        );
    }
}
