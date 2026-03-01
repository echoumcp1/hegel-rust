use super::{BasicGenerator, Generate, TestCaseData};
use crate::cbor_utils::{cbor_map, map_insert};
use ciborium::Value;

pub struct TextGenerator {
    min_size: usize,
    max_size: Option<usize>,
}

impl TextGenerator {
    pub fn with_min_size(mut self, min: usize) -> Self {
        self.min_size = min;
        self
    }

    pub fn with_max_size(mut self, max: usize) -> Self {
        self.max_size = Some(max);
        self
    }

    fn build_schema(&self) -> Value {
        let mut schema = cbor_map! {
            "type" => "string",
            "min_size" => self.min_size as u64
        };

        if let Some(max) = self.max_size {
            map_insert(&mut schema, "max_size", Value::from(max as u64));
        }

        schema
    }
}

impl Generate<String> for TextGenerator {
    fn do_draw(&self, data: &TestCaseData) -> String {
        data.generate_from_schema(&self.build_schema())
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, String>> {
        Some(BasicGenerator::new(self.build_schema(), |raw| {
            super::deserialize_value(raw)
        }))
    }
}

pub fn text() -> TextGenerator {
    TextGenerator {
        min_size: 0,
        max_size: None,
    }
}

pub struct RegexGenerator {
    pattern: String,
    fullmatch: bool,
}

impl RegexGenerator {
    /// Require the entire string to match the pattern, not just contain a match.
    pub fn fullmatch(mut self) -> Self {
        self.fullmatch = true;
        self
    }

    fn build_schema(&self) -> Value {
        cbor_map! {
            "type" => "regex",
            "pattern" => self.pattern.as_str(),
            "fullmatch" => self.fullmatch
        }
    }
}

impl Generate<String> for RegexGenerator {
    fn do_draw(&self, data: &TestCaseData) -> String {
        data.generate_from_schema(&self.build_schema())
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, String>> {
        Some(BasicGenerator::new(self.build_schema(), |raw| {
            super::deserialize_value(raw)
        }))
    }
}

/// Generate strings that contain a match for the given regex pattern.
///
/// Use `.fullmatch()` to require the entire string to match.
pub fn from_regex(pattern: &str) -> RegexGenerator {
    RegexGenerator {
        pattern: pattern.to_string(),
        fullmatch: false,
    }
}

/// Generator for binary data (byte sequences).
pub struct BinaryGenerator {
    min_size: usize,
    max_size: Option<usize>,
}

impl BinaryGenerator {
    /// Set the minimum size in bytes.
    pub fn with_min_size(mut self, min: usize) -> Self {
        self.min_size = min;
        self
    }

    /// Set the maximum size in bytes.
    pub fn with_max_size(mut self, max: usize) -> Self {
        self.max_size = Some(max);
        self
    }

    fn build_schema(&self) -> Value {
        let mut schema = cbor_map! {
            "type" => "binary",
            "min_size" => self.min_size as u64
        };

        if let Some(max) = self.max_size {
            map_insert(&mut schema, "max_size", Value::from(max as u64));
        }

        schema
    }
}

fn parse_binary(raw: Value) -> Vec<u8> {
    match raw {
        Value::Bytes(bytes) => bytes,
        _ => panic!(
            "Expected CBOR byte string from binary schema, got {:?}",
            raw
        ),
    }
}

impl Generate<Vec<u8>> for BinaryGenerator {
    fn do_draw(&self, data: &TestCaseData) -> Vec<u8> {
        parse_binary(data.generate_raw(&self.build_schema()))
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, Vec<u8>>> {
        Some(BasicGenerator::new(self.build_schema(), parse_binary))
    }
}

/// Generate binary data (byte sequences).
///
/// # Example
///
/// ```no_run
/// use hegel::generators::{self, Generate};
///
/// // Generate any byte sequence
/// let gen = generators::binary();
///
/// // Generate 16-32 bytes
/// let gen = generators::binary().with_min_size(16).with_max_size(32);
/// ```
pub fn binary() -> BinaryGenerator {
    BinaryGenerator {
        min_size: 0,
        max_size: None,
    }
}
