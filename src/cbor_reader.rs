//! CBOR-to-Value parser that preserves bignum tags (2/3) as `Value::Tag`
//! instead of auto-decoding into `Value::Integer` (which overflows for >i128).

use ciborium::Value;
use std::io::{self, Read};

pub fn read_value(r: &mut impl Read) -> io::Result<Value> {
    let initial = read_u8(r)?;
    decode(r, initial)
}

fn read_u8(r: &mut impl Read) -> io::Result<u8> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_argument(r: &mut impl Read, additional: u8) -> io::Result<u64> {
    match additional {
        0..=23 => Ok(additional as u64),
        24 => Ok(read_u8(r)? as u64),
        25 => {
            let mut b = [0u8; 2];
            r.read_exact(&mut b)?;
            Ok(u16::from_be_bytes(b) as u64)
        }
        26 => {
            let mut b = [0u8; 4];
            r.read_exact(&mut b)?;
            Ok(u32::from_be_bytes(b) as u64)
        }
        27 => {
            let mut b = [0u8; 8];
            r.read_exact(&mut b)?;
            Ok(u64::from_be_bytes(b))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid CBOR additional info: {additional}"),
        )),
    }
}

fn read_raw_bytes(r: &mut impl Read, additional: u8) -> io::Result<Vec<u8>> {
    if additional == 31 {
        let mut buf = Vec::new();
        loop {
            let peek = read_u8(r)?;
            if peek == 0xff {
                break;
            }
            let len = read_argument(r, peek & 0x1f)? as usize;
            let mut chunk = vec![0u8; len];
            r.read_exact(&mut chunk)?;
            buf.extend_from_slice(&chunk);
        }
        Ok(buf)
    } else {
        let len = read_argument(r, additional)? as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        Ok(buf)
    }
}

fn read_indefinite<R: Read, T>(
    r: &mut R,
    mut item: impl FnMut(&mut R, u8) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let mut out = Vec::new();
    loop {
        let peek = read_u8(r)?;
        if peek == 0xff {
            return Ok(out);
        }
        out.push(item(r, peek)?);
    }
}

/// Decode a single CBOR value given its already-read initial byte.
fn decode(r: &mut impl Read, initial: u8) -> io::Result<Value> {
    let major = initial >> 5;
    let additional = initial & 0x1f;

    match major {
        0 => {
            let v = read_argument(r, additional)?;
            Ok(if v <= i64::MAX as u64 {
                Value::from(v as i64)
            } else {
                Value::Integer(ciborium::value::Integer::from(v))
            })
        }
        1 => {
            let v = read_argument(r, additional)?;
            Ok(if v <= i64::MAX as u64 {
                Value::from(-(v as i64) - 1)
            } else {
                let bytes = v.to_be_bytes();
                let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
                Value::Tag(3, Box::new(Value::Bytes(bytes[start..].to_vec())))
            })
        }
        2 => Ok(Value::Bytes(read_raw_bytes(r, additional)?)),
        3 => {
            let bytes = read_raw_bytes(r, additional)?;
            let s = String::from_utf8(bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(Value::Text(s))
        }
        4 => {
            if additional == 31 {
                Ok(Value::Array(read_indefinite(r, decode)?))
            } else {
                let n = read_argument(r, additional)? as usize;
                (0..n)
                    .map(|_| read_value(r))
                    .collect::<io::Result<Vec<_>>>()
                    .map(Value::Array)
            }
        }
        5 => {
            if additional == 31 {
                let items = read_indefinite(r, |r, peek| Ok((decode(r, peek)?, read_value(r)?)))?;
                Ok(Value::Map(items))
            } else {
                let n = read_argument(r, additional)? as usize;
                let mut entries = Vec::with_capacity(n);
                for _ in 0..n {
                    entries.push((read_value(r)?, read_value(r)?));
                }
                Ok(Value::Map(entries))
            }
        }
        6 => {
            let tag = read_argument(r, additional)?;
            let inner = read_value(r)?;
            Ok(Value::Tag(tag, Box::new(inner)))
        }
        7 => match additional {
            20 => Ok(Value::Bool(false)),
            21 => Ok(Value::Bool(true)),
            22 | 23 => Ok(Value::Null), // null and undefined map to same
            24 => Ok(match read_u8(r)? {
                20 => Value::Bool(false),
                21 => Value::Bool(true),
                _ => Value::Null,
            }),
            25 => {
                let mut b = [0u8; 2];
                r.read_exact(&mut b)?;
                Ok(Value::Float(f64::from(half::f16::from_be_bytes(b))))
            }
            26 => {
                let mut b = [0u8; 4];
                r.read_exact(&mut b)?;
                Ok(Value::Float(f32::from_be_bytes(b) as f64))
            }
            27 => {
                let mut b = [0u8; 8];
                r.read_exact(&mut b)?;
                Ok(Value::Float(f64::from_be_bytes(b)))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported simple value: additional={additional}"),
            )),
        },
        _ => unreachable!("CBOR major type > 7"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(value: &Value) -> Value {
        let mut buf = Vec::new();
        ciborium::into_writer(value, &mut buf).unwrap();
        read_value(&mut &buf[..]).unwrap()
    }

    #[test]
    fn test_integers() {
        assert_eq!(roundtrip(&Value::from(0)), Value::from(0));
        assert_eq!(roundtrip(&Value::from(42)), Value::from(42));
        assert_eq!(roundtrip(&Value::from(-1)), Value::from(-1));
        assert_eq!(roundtrip(&Value::from(i64::MAX)), Value::from(i64::MAX));
        assert_eq!(roundtrip(&Value::from(i64::MIN)), Value::from(i64::MIN));
    }

    #[test]
    fn test_strings() {
        let v = Value::Text("hello".into());
        assert_eq!(roundtrip(&v), v);
    }

    #[test]
    fn test_bytes() {
        let v = Value::Bytes(vec![1, 2, 3]);
        assert_eq!(roundtrip(&v), v);
    }

    #[test]
    fn test_array() {
        let v = Value::Array(vec![Value::from(1), Value::Text("two".into())]);
        assert_eq!(roundtrip(&v), v);
    }

    #[test]
    fn test_map() {
        let v = Value::Map(vec![
            (Value::Text("key".into()), Value::from(42)),
            (Value::Text("other".into()), Value::Bool(true)),
        ]);
        assert_eq!(roundtrip(&v), v);
    }

    #[test]
    fn test_bignum_tags_preserved() {
        let big_bytes = {
            let mut b = vec![1u8];
            b.extend(std::iter::repeat(0u8).take(16));
            b
        };
        let v = Value::Tag(2, Box::new(Value::Bytes(big_bytes.clone())));
        let mut buf = Vec::new();
        ciborium::into_writer(&v, &mut buf).unwrap();
        let parsed = read_value(&mut &buf[..]).unwrap();
        assert_eq!(parsed, Value::Tag(2, Box::new(Value::Bytes(big_bytes))));
    }

    #[test]
    fn test_floats() {
        let v = Value::Float(3.14);
        let parsed = roundtrip(&v);
        if let Value::Float(f) = parsed {
            assert!((f - 3.14).abs() < 1e-10);
        } else {
            panic!("expected float");
        }
    }

    #[test]
    fn test_booleans_and_null() {
        assert_eq!(roundtrip(&Value::Bool(true)), Value::Bool(true));
        assert_eq!(roundtrip(&Value::Bool(false)), Value::Bool(false));
        assert_eq!(roundtrip(&Value::Null), Value::Null);
    }

    #[test]
    fn test_nested_map_with_bignum() {
        let big_bytes = vec![0xFF; 17];
        let v = Value::Map(vec![(
            Value::Text("result".into()),
            Value::Tag(2, Box::new(Value::Bytes(big_bytes.clone()))),
        )]);
        let mut buf = Vec::new();
        ciborium::into_writer(&v, &mut buf).unwrap();
        let parsed = read_value(&mut &buf[..]).unwrap();
        assert_eq!(
            parsed,
            Value::Map(vec![(
                Value::Text("result".into()),
                Value::Tag(2, Box::new(Value::Bytes(big_bytes))),
            )])
        );
    }
}
