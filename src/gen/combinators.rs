use super::{discardable_group, generate_from_schema, group, integers, labels, Generate};
use crate::cbor_helpers::{cbor_array, cbor_map, cbor_serialize};
use ciborium::Value;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct Mapped<T, U, F, G> {
    pub(crate) source: G,
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<(T, U)>,
}

impl<T, U, F, G> Generate<U> for Mapped<T, U, F, G>
where
    G: Generate<T>,
    F: Fn(T) -> U + Send + Sync,
{
    fn generate(&self) -> U {
        group(labels::MAPPED, || (self.f)(self.source.generate()))
    }

    fn schema(&self) -> Option<Value> {
        None
    }
}

// Safety: Mapped is Send+Sync if its components are
unsafe impl<T, U, F, G> Send for Mapped<T, U, F, G>
where
    G: Send,
    F: Send,
{
}

unsafe impl<T, U, F, G> Sync for Mapped<T, U, F, G>
where
    G: Sync,
    F: Sync,
{
}

pub struct FlatMapped<T, U, G2, F, G1> {
    pub(crate) source: G1,
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<(T, U, G2)>,
}

impl<T, U, G2, F, G1> Generate<U> for FlatMapped<T, U, G2, F, G1>
where
    G1: Generate<T>,
    G2: Generate<U>,
    F: Fn(T) -> G2 + Send + Sync,
{
    fn generate(&self) -> U {
        group(labels::FLAT_MAP, || {
            let intermediate = self.source.generate();
            let next_gen = (self.f)(intermediate);
            next_gen.generate()
        })
    }

    fn schema(&self) -> Option<Value> {
        None
    }
}

unsafe impl<T, U, G2, F, G1> Send for FlatMapped<T, U, G2, F, G1>
where
    G1: Send,
    F: Send,
{
}

unsafe impl<T, U, G2, F, G1> Sync for FlatMapped<T, U, G2, F, G1>
where
    G1: Sync,
    F: Sync,
{
}

pub struct Filtered<T, F, G> {
    pub(crate) source: G,
    pub(crate) predicate: F,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T, F, G> Generate<T> for Filtered<T, F, G>
where
    G: Generate<T>,
    F: Fn(&T) -> bool + Send + Sync,
{
    fn generate(&self) -> T {
        for _ in 0..3 {
            if let Some(value) = discardable_group(labels::FILTER, || {
                let value = self.source.generate();
                if (self.predicate)(&value) {
                    Some(value)
                } else {
                    None
                }
            }) {
                return value;
            }
        }
        crate::assume(false);
        unreachable!()
    }

    fn schema(&self) -> Option<Value> {
        None
    }
}

unsafe impl<T, F, G> Send for Filtered<T, F, G>
where
    G: Send,
    F: Send,
{
}

unsafe impl<T, F, G> Sync for Filtered<T, F, G>
where
    G: Sync,
    F: Sync,
{
}

// ============================================================================
// BoxedGenerator - Type-erased generator
// ============================================================================

/// A type-erased generator with a lifetime parameter.
///
/// This is useful for storing generators of different concrete types
/// in collections or struct fields.
///
/// Create a `BoxedGenerator` by calling `.boxed()` on any generator.
///
/// The lifetime `'a` represents the minimum lifetime of any borrowed data
/// in the generator. Use `'static` for generators that own all their data.
/// For generators that borrow data, the lifetime will match the borrow.
pub struct BoxedGenerator<'a, T> {
    pub(crate) inner: Arc<dyn Generate<T> + Send + Sync + 'a>,
}

impl<'a, T> Clone for BoxedGenerator<'a, T> {
    fn clone(&self) -> Self {
        BoxedGenerator {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<'a, T> Generate<T> for BoxedGenerator<'a, T> {
    fn generate(&self) -> T {
        self.inner.generate()
    }

    fn schema(&self) -> Option<Value> {
        self.inner.schema()
    }

    /// Returns self without re-wrapping.
    fn boxed<'b>(self) -> BoxedGenerator<'b, T>
    where
        Self: Sized + Send + Sync + 'b,
    {
        BoxedGenerator { inner: self.inner }
    }
}

pub struct SampledFromGenerator<T> {
    elements: Vec<T>,
}

impl<T: Clone + Send + Sync + serde::Serialize> Generate<T> for SampledFromGenerator<T> {
    fn generate(&self) -> T {
        crate::assume(!self.elements.is_empty());

        // Check if elements are primitive enough for sampled_from schema
        if let Some(schema) = self.schema() {
            let value: Value = generate_from_schema(&schema);
            // Find matching element by comparing serialized forms
            for elem in &self.elements {
                let elem_cbor = cbor_serialize(elem);
                if elem_cbor == value {
                    return elem.clone();
                }
            }
            panic!(
                "hegel: sampled_from received value not in elements list: {:?}",
                value
            );
        } else {
            // Generate index and pick
            let idx_gen = integers::<usize>()
                .with_min(0)
                .with_max(self.elements.len() - 1);
            let idx = idx_gen.generate();
            self.elements[idx].clone()
        }
    }

    fn schema(&self) -> Option<Value> {
        // Only use sampled_from schema for CBOR-primitive types
        let cbor_values: Vec<Value> = self.elements.iter().map(|e| cbor_serialize(e)).collect();

        // Check if all values are primitives (not maps/arrays)
        let all_primitive = cbor_values.iter().all(|v| {
            matches!(
                v,
                Value::Null | Value::Bool(_) | Value::Integer(_) | Value::Float(_) | Value::Text(_)
            )
        });

        if all_primitive {
            Some(cbor_map! {"sampled_from" => Value::Array(cbor_values)})
        } else {
            None
        }
    }
}

pub fn sampled_from<T: Clone + Send + Sync + serde::Serialize>(
    elements: Vec<T>,
) -> SampledFromGenerator<T> {
    SampledFromGenerator { elements }
}

pub struct SampledFromSliceGenerator<'a, T> {
    elements: &'a [T],
}

impl<'a, T: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned> Generate<T>
    for SampledFromSliceGenerator<'a, T>
{
    fn generate(&self) -> T {
        crate::assume(!self.elements.is_empty());

        if let Some(schema) = self.schema() {
            generate_from_schema(&schema)
        } else {
            // Compositional fallback
            group(labels::SAMPLED_FROM, || {
                let idx = integers::<usize>()
                    .with_min(0)
                    .with_max(self.elements.len() - 1)
                    .generate();
                self.elements[idx].clone()
            })
        }
    }

    fn schema(&self) -> Option<Value> {
        let cbor_values: Vec<Value> = self.elements.iter().map(|e| cbor_serialize(e)).collect();
        Some(cbor_map! {"sampled_from" => Value::Array(cbor_values)})
    }
}

// Safety: SampledFromSliceGenerator is Send+Sync if T is Send+Sync
unsafe impl<'a, T: Send + Sync> Send for SampledFromSliceGenerator<'a, T> {}
unsafe impl<'a, T: Send + Sync> Sync for SampledFromSliceGenerator<'a, T> {}

/// Sample uniformly from a borrowed slice.
///
/// This allows creating generators that borrow from local data,
/// enabling non-`'static` lifetimes.
///
/// # Example
///
/// ```no_run
/// use hegel::gen::{self, Generate, BoxedGenerator};
///
/// let choices = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
/// let gen: BoxedGenerator<'_, String> = gen::sampled_from_slice(&choices).boxed();
/// let value = gen.generate();
/// ```
pub fn sampled_from_slice<
    T: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned,
>(
    elements: &[T],
) -> SampledFromSliceGenerator<'_, T> {
    SampledFromSliceGenerator { elements }
}

pub struct OneOfGenerator<'a, T> {
    generators: Vec<BoxedGenerator<'a, T>>,
}

impl<'a, T: serde::de::DeserializeOwned> Generate<T> for OneOfGenerator<'a, T> {
    fn generate(&self) -> T {
        crate::assume(!self.generators.is_empty());

        if let Some(schema) = self.schema() {
            generate_from_schema(&schema)
        } else {
            // Generate index and delegate
            group(labels::ONE_OF, || {
                let idx = integers::<usize>()
                    .with_min(0)
                    .with_max(self.generators.len() - 1)
                    .generate();
                self.generators[idx].generate()
            })
        }
    }

    fn schema(&self) -> Option<Value> {
        let schemas: Option<Vec<Value>> = self.generators.iter().map(|g| g.schema()).collect();

        schemas.map(|s| cbor_map! {"one_of" => Value::Array(s)})
    }
}

/// Choose from multiple generators of the same type.
///
/// For a more convenient syntax, use the `one_of!` macro instead.
pub fn one_of<'a, T>(generators: Vec<BoxedGenerator<'a, T>>) -> OneOfGenerator<'a, T> {
    OneOfGenerator { generators }
}

/// Choose from multiple generators of the same type.
///
/// This macro automatically boxes each generator, providing a more ergonomic
/// syntax than calling [`one_of`] directly.
///
/// # Example
///
/// ```no_run
/// use hegel::gen::{self, Generate};
///
/// let gen = hegel::one_of!(
///     gen::integers::<i32>().with_min(0).with_max(10),
///     gen::integers::<i32>().with_min(100).with_max(110),
/// );
/// let value = gen.generate();
/// ```
#[macro_export]
macro_rules! one_of {
    ($($gen:expr),+ $(,)?) => {
        $crate::gen::one_of(vec![
            $($crate::gen::Generate::boxed($gen)),+
        ])
    };
}

pub struct OptionalGenerator<G> {
    inner: G,
}

impl<T, G> Generate<Option<T>> for OptionalGenerator<G>
where
    G: Generate<T>,
    T: serde::de::DeserializeOwned,
{
    fn generate(&self) -> Option<T> {
        if let Some(inner_schema) = self.inner.schema() {
            let schema = cbor_map! {
                "one_of" => cbor_array![
                    cbor_map!{"type" => "null"},
                    inner_schema
                ]
            };
            generate_from_schema(&schema)
        } else {
            // Compositional fallback
            group(labels::OPTIONAL, || {
                let is_some: bool = generate_from_schema(&cbor_map! {"type" => "boolean"});
                if is_some {
                    Some(self.inner.generate())
                } else {
                    None
                }
            })
        }
    }

    fn schema(&self) -> Option<Value> {
        let inner_schema = self.inner.schema()?;
        Some(cbor_map! {
            "one_of" => cbor_array![
                cbor_map!{"type" => "null"},
                inner_schema
            ]
        })
    }
}

pub fn optional<T, G: Generate<T>>(inner: G) -> OptionalGenerator<G> {
    OptionalGenerator { inner }
}
