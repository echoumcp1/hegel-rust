RELEASE_TYPE: patch

Bump our pinned hegel-core to [0.4.14](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.14), incorporating the following changes:

> This patch removes the unused Unix socket transport from the `hegel` server. The server now always communicates with its client over stdin/stdout, matching how all current libraries spawn it.
>
> — [v0.4.8](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.8)

> This release adds a `command_prefix` argument to `run_conformance_tests` to control how conformance tests are run.
>
> — [v0.4.9](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.9)

> Add fraction and complex number schema types.
>
> — [v0.4.10](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.10)

> This release adds a `skip_unique` parameter to `ListConformance`.
>
> — [v0.4.11](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.11)

> Removes CBOR tagging from fraction and complex numbers.
>
> — [v0.4.12](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.12)

> This release tweaks how our conformance tests write metrics.
>
> — [v0.4.13](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.13)

> Pin our dependencies to below their next major version.
>
> — [v0.4.14](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.14)
