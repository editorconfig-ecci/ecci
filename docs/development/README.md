# Development documentation

This area contains human-facing contributor and maintainer workflows when they
need more detail than the repository's standard build and test commands.

## Build and test

Building from source requires a current stable Rust toolchain with Cargo and
the EditorConfig Core C development library (`libeditorconfig`). The supplied
development container includes these dependencies.

Run the workspace test suite with:

```sh
cargo test --workspace
```

Tests use fixtures under [`testdata`](../../testdata). Some tests are
intentionally ignored to record known defects; inspect the test output before
treating the suite as a release gate.

Before contributing, run the checks used by continuous integration:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --release
cargo clippy
```
