RELEASE_TYPE: patch

Bump our pinned hegel-core to [0.4.4](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.4), incorporating the following changes:

> This patch adds a new `OneOfConformance` test, for the `one_of` generator.
>
> This patch also adds recommended integer bound constants (`INT32_MIN`, `INT32_MAX`, `INT64_MIN`, `INT64_MAX`, `BIGINT_MIN`, `BIGINT_MAX`) for use in conformance test setup. Languages with arbitrary-precision integers should use the `BIGINT` bounds to exercise CBOR bignum tag decoding, which is not triggered by the narrower ranges most implementations currently use.
>
> — [v0.4.3](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.3)

> This release is in support of getting hegel libraries working on Windows. It mostly fixes issues affecting the conformance testing.
>
> Windows support still won't work in individual libraries until they also do work to support it.
>
> — [v0.4.4](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.4)
