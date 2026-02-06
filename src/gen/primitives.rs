use super::{generate_from_schema, Generate};
use crate::cbor_helpers::{cbor_map, cbor_serialize};
use ciborium::Value;

pub fn unit() -> JustGenerator<()> {
    just(())
}

pub struct JustGenerator<T> {
    value: T,
}

impl<T: Clone + Send + Sync + serde::Serialize> Generate<T> for JustGenerator<T> {
    fn generate(&self) -> T {
        self.value.clone()
    }

    fn schema(&self) -> Option<Value> {
        Some(cbor_map! {"const" => cbor_serialize(&self.value)})
    }
}

pub fn just<T: Clone + Send + Sync + serde::Serialize>(value: T) -> JustGenerator<T> {
    JustGenerator { value }
}

pub struct JustAnyGenerator<T> {
    value: T,
}

impl<T: Clone + Send + Sync> Generate<T> for JustAnyGenerator<T> {
    fn generate(&self) -> T {
        self.value.clone()
    }

    fn schema(&self) -> Option<Value> {
        None
    }
}
pub fn just_any<T: Clone + Send + Sync>(value: T) -> JustAnyGenerator<T> {
    JustAnyGenerator { value }
}

pub struct BoolGenerator;

impl Generate<bool> for BoolGenerator {
    fn generate(&self) -> bool {
        generate_from_schema(&cbor_map! {"type" => "boolean"})
    }

    fn schema(&self) -> Option<Value> {
        Some(cbor_map! {"type" => "boolean"})
    }
}

pub fn booleans() -> BoolGenerator {
    BoolGenerator
}
