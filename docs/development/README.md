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

The initial-release CLI and Docker Action acceptance fixture is under
[`crates/ecci/tests/fixtures/release-semantics`](../../crates/ecci/tests/fixtures/release-semantics).
It covers UTF-16 selection, binary exclusion, `.ecciignore` force-check rules,
nested `.editorconfig` resolution, reporting suppression, and violation-status
remapping. Run its CLI, Action-mode, metadata, and container-entrypoint tests
with:

```sh
cargo test --package ecci --test cli --test action
```

Build the same Docker Action definition used by GitHub with:

```sh
docker build --tag ecci-action:test .
```

JSON and Static Analysis Results Interchange Format (SARIF) output are deferred
and are not part of the initial-release test matrix.

Before contributing, run the checks used by continuous integration:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --release
cargo clippy
```
