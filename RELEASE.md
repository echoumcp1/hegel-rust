RELEASE_TYPE: minor

* Renames `from_type` to `default` (to be used as `generators::default`)
* Makes `default::<T>` always return a `BoxedGenerator<T>`. This means you can no longer do things like `default::<i32>.min_value(0)`, but also means that the `T` parameter can be reliably inferred so `default()` will work without having to be `default::<T>()` in many more cases.
