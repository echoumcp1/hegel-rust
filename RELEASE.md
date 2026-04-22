RELEASE_TYPE: patch

This patch adds `generators::deferred()`, which creates a generator that can be declared before it is defined. This enables forward references, which are needed for defining mutually recursive or self-recursive generators.

```rust
use hegel::generators::{self as gs, Generator};

let x = gs::deferred::<i32>();
let y = gs::deferred::<i32>();

y.set(hegel::one_of!(gs::integers::<i32>().min_value(0).max_value(10), x.clone()).boxed());
x.set(hegel::one_of!(gs::integers::<i32>().min_value(100).max_value(110), y.clone()).boxed());
```
