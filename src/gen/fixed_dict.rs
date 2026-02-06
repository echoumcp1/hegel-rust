use super::{generate_from_schema, group, labels, BoxedGenerator, Generate};
use crate::cbor_helpers::{cbor_map, cbor_serialize};
use ciborium::Value;
use std::marker::PhantomData;
use std::sync::Arc;

pub(crate) struct MappedToValue<T, G> {
    inner: G,
    _phantom: PhantomData<T>,
}

impl<T: serde::Serialize, G: Generate<T>> Generate<Value> for MappedToValue<T, G> {
    fn generate(&self) -> Value {
        cbor_serialize(&self.inner.generate())
    }

    fn schema(&self) -> Option<Value> {
        self.inner.schema()
    }
}

unsafe impl<T, G: Send> Send for MappedToValue<T, G> {}
unsafe impl<T, G: Sync> Sync for MappedToValue<T, G> {}

pub struct FixedDictBuilder<'a> {
    fields: Vec<(String, BoxedGenerator<'a, Value>)>,
}

impl<'a> FixedDictBuilder<'a> {
    pub fn field<T, G>(mut self, name: &str, gen: G) -> Self
    where
        G: Generate<T> + Send + Sync + 'a,
        T: serde::Serialize + 'a,
    {
        let boxed = BoxedGenerator {
            inner: Arc::new(MappedToValue {
                inner: gen,
                _phantom: PhantomData::<T>,
            }),
        };
        self.fields.push((name.to_string(), boxed));
        self
    }

    pub fn build(self) -> FixedDictGenerator<'a> {
        FixedDictGenerator {
            fields: self.fields,
        }
    }
}

pub struct FixedDictGenerator<'a> {
    fields: Vec<(String, BoxedGenerator<'a, Value>)>,
}

impl<'a> Generate<Value> for FixedDictGenerator<'a> {
    fn generate(&self) -> Value {
        if let Some(schema) = self.schema() {
            let values: Vec<Value> = generate_from_schema(&schema);
            // Convert tuple back to object (map)
            let entries: Vec<(Value, Value)> = self
                .fields
                .iter()
                .zip(values)
                .map(|((name, _), value)| (Value::Text(name.clone()), value))
                .collect();
            Value::Map(entries)
        } else {
            // Compositional fallback
            group(labels::FIXED_DICT, || {
                let entries: Vec<(Value, Value)> = self
                    .fields
                    .iter()
                    .map(|(name, gen)| (Value::Text(name.clone()), gen.generate()))
                    .collect();
                Value::Map(entries)
            })
        }
    }

    fn schema(&self) -> Option<Value> {
        let mut elements = Vec::new();

        for (_, gen) in &self.fields {
            let field_schema = gen.schema()?;
            elements.push(field_schema);
        }

        Some(cbor_map! {
            "type" => "tuple",
            "elements" => Value::Array(elements)
        })
    }
}

/// Create a generator for dictionaries with fixed keys.
///
/// # Example
///
/// ```no_run
/// use hegel::gen::{self, Generate};
///
/// let gen = gen::fixed_dicts()
///     .field("name", gen::text())
///     .field("age", gen::integers::<u32>())
///     .build();
/// ```
pub fn fixed_dicts<'a>() -> FixedDictBuilder<'a> {
    FixedDictBuilder { fields: Vec::new() }
}
