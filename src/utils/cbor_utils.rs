use ciborium::Value;
use ciborium_ll::{Decoder, Header};
use std::io::{self, Read};

/// Build a `ciborium::Value::Map`:
///
/// ```ignore
/// let schema = cbor_map!{
///     "type" => "integer",
///     "min_value" => 0,
///     "max_value" => 100
/// };
/// ```
macro_rules! cbor_map {
    ($($key:expr => $value:expr),* $(,)?) => {
        ciborium::Value::Map(vec![
            $((
                ciborium::Value::Text(String::from($key)),
                ciborium::Value::from($value),
            )),*
        ])
    };
}

/// Build a `ciborium::Value::Array`:
///
/// ```ignore
/// let elements = cbor_array![schema1, schema2];
/// ```
macro_rules! cbor_array {
    ($($value:expr),* $(,)?) => {
        ciborium::Value::Array(vec![$($value),*])
    };
}

pub(crate) use cbor_array;
pub(crate) use cbor_map;

pub fn map_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let Value::Map(entries) = value else {
        panic!("expected Value::Map, got {value:?}"); // nocov
    };
    for (k, v) in entries {
        let Value::Text(s) = k else {
            panic!("expected Value::Text, got {k:?}"); // nocov
        };
        if s == key {
            return Some(v);
        }
    }
    None
}

pub fn map_insert(value: &mut Value, key: &str, val: impl Into<Value>) {
    let Value::Map(entries) = value else {
        panic!("expected Value::Map, got {value:?}"); // nocov
    };
    let val = val.into();
    for (k, v) in entries.iter_mut() {
        let Value::Text(s) = k else {
            panic!("expected Value::Text, got {k:?}"); // nocov
        };
        if s == key {
            *v = val;
            return;
        }
    }
    entries.push((Value::Text(String::from(key)), val));
}

// merge the keys of two maps. If both `target` and `source` contain the same key,
// prefer `source`.
pub fn map_extend(target: &mut Value, source: Value) {
    let Value::Map(source_entries) = source else {
        panic!("expected Value::Map, got {source:?}");
    };
    for (k, v) in source_entries {
        let Value::Text(ref key) = k else {
            panic!("expected Value::Text, got {k:?}");
        };
        map_insert(target, key, v);
    }
}

pub fn as_text(value: &Value) -> Option<&str> {
    match value {
        Value::Text(s) => Some(s),
        _ => None, // nocov
    }
}

pub fn as_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Integer(i) => u64::try_from(i128::from(*i)).ok(),
        _ => None, // nocov
    }
}

pub fn as_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(b) => Some(*b),
        _ => None,
    }
}

pub fn cbor_serialize<T: serde::Serialize>(value: &T) -> Value {
    let mut bytes = Vec::new();
    ciborium::into_writer(value, &mut bytes).expect("CBOR serialization failed");
    ciborium::from_reader(&bytes[..]).expect("CBOR deserialization failed")
}

pub fn read_value(r: &mut impl Read) -> io::Result<Value> {
    let mut decoder = Decoder::from(r);
    pull_value(&mut decoder)
}

fn pull_value<R: Read>(decoder: &mut Decoder<R>) -> io::Result<Value> {
    let header = decoder.pull().map_err(map_decoder_err)?;
    decode_header(decoder, header)
}

fn decode_header<R: Read>(decoder: &mut Decoder<R>, header: Header) -> io::Result<Value> {
    match header {
        Header::Positive(v) => Ok(Value::Integer(ciborium::value::Integer::from(v))),
        Header::Negative(v) => Ok(if v <= i64::MAX as u64 {
            Value::from(-(v as i64) - 1)
        } else {
            // cbor2 doesn't tag negative ints with BIGNEG (tag 3) unless it's less than -2^64
            // this else branch is required to prevent a crash from negating with overflow
            let bytes = v.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
            Value::Tag(3, Box::new(Value::Bytes(bytes[start..].to_vec())))
        }),
        Header::Float(f) => Ok(Value::Float(f)),
        Header::Bytes(len) => Ok(Value::Bytes(read_segmented_bytes(decoder, len)?)),
        Header::Text(len) => Ok(Value::Text(read_segmented_text(decoder, len)?)),
        Header::Array(len) => Ok(Value::Array(read_array(decoder, len)?)),
        Header::Map(len) => Ok(Value::Map(read_map(decoder, len)?)),
        Header::Tag(tag) => {
            let inner = pull_value(decoder)?;
            Ok(Value::Tag(tag, Box::new(inner))) // bigint are parsed here
        }
        Header::Simple(simple) => Ok(match simple {
            ciborium_ll::simple::FALSE => Value::Bool(false),
            ciborium_ll::simple::TRUE => Value::Bool(true),
            ciborium_ll::simple::NULL => Value::Null,
            _ => panic!("unexpected simple value: {simple:?}"),
        }),
        Header::Break => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected CBOR break",
        )),
    }
}

fn read_array<R: Read>(decoder: &mut Decoder<R>, len: Option<usize>) -> io::Result<Vec<Value>> {
    if let Some(n) = len {
        return (0..n).map(|_| pull_value(decoder)).collect();
    }
    let mut items = Vec::new();
    loop {
        let header = decoder.pull().map_err(map_decoder_err)?;
        if matches!(header, Header::Break) {
            return Ok(items);
        }
        items.push(decode_header(decoder, header)?);
    }
}

fn read_map<R: Read>(
    decoder: &mut Decoder<R>,
    len: Option<usize>,
) -> io::Result<Vec<(Value, Value)>> {
    if let Some(n) = len {
        let mut entries = Vec::with_capacity(n);
        for _ in 0..n {
            entries.push((pull_value(decoder)?, pull_value(decoder)?));
        }
        return Ok(entries);
    }
    let mut entries = Vec::new();
    loop {
        let header = decoder.pull().map_err(map_decoder_err)?;
        if matches!(header, Header::Break) {
            return Ok(entries);
        }
        let key = decode_header(decoder, header)?;
        let val = pull_value(decoder)?;
        entries.push((key, val));
    }
}

fn read_segmented_bytes<R: Read>(
    decoder: &mut Decoder<R>,
    len: Option<usize>,
) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut segments = decoder.bytes(len);
    while let Some(mut segment) = segments.pull().map_err(map_decoder_err)? {
        while let Some(part) = segment.pull(&mut chunk).map_err(map_decoder_err)? {
            buf.extend_from_slice(part);
        }
    }
    Ok(buf)
}

fn read_segmented_text<R: Read>(
    decoder: &mut Decoder<R>,
    len: Option<usize>,
) -> io::Result<String> {
    let mut buf = String::new();
    let mut chunk = [0u8; 4096];
    let mut segments = decoder.text(len);
    while let Some(mut segment) = segments.pull().map_err(map_decoder_err)? {
        while let Some(part) = segment.pull(&mut chunk).map_err(map_decoder_err)? {
            buf.push_str(part);
        }
    }
    Ok(buf)
}

fn map_decoder_err<E: std::fmt::Debug>(e: ciborium_ll::Error<E>) -> io::Error {
    match e {
        ciborium_ll::Error::Io(io_err) => io::Error::other(format!("{io_err:?}")),
        ciborium_ll::Error::Syntax(offset) => io::Error::new(
            io::ErrorKind::InvalidData,
            format!("CBOR syntax error at offset {offset}"),
        ),
    }
}

#[cfg(test)]
#[path = "../../tests/embedded/cbor_utils_tests.rs"]
mod tests;
