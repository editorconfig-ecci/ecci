# ecci

`ecci` is the end-user command-line interface for checking specified files and
directories against the applicable [EditorConfig](https://editorconfig.org/)
settings. Its goal is to make `.editorconfig` conformance easy to run locally
and in automation.

> **Current status:** the command-line interface is under active development.
> It does not yet accept paths, traverse directories, or report check results.
> The only current executable behavior is described in
> [Running the current CLI](#running-the-current-cli). The checking logic below
> is implemented in internal libraries and is not yet wired to the CLI.

## Prerequisites

Building from source requires:

- A current stable [Rust toolchain](https://www.rust-lang.org/tools/install)
  with Cargo.
- The EditorConfig Core C development library, because the project links to
  `libeditorconfig`.

On Debian or Ubuntu, install the native dependency with:

```sh
sudo apt-get install libeditorconfig-dev
```

On Alpine, install:

```sh
apk add editorconfig-dev
```

The supplied development container includes Rust and `libeditorconfig-dev`.

## Build

Clone the repository and build the end-user CLI:

```sh
git clone https://github.com/editorconfig-ecci/ecci.git
cd ecci
cargo build --release --package ecci
```

The resulting executable is `target/release/ecci`.

## Running the current CLI

Run the command from the repository root:

```sh
cargo run --package ecci
```

At present, this reads the repository-root `Cargo.toml`, resolves its
EditorConfig settings, and prints its `indent_style`. With this repository's
current configuration, the output is:

```text
Cargo.toml indent_style:Space
```

There are no command-line options or path arguments yet. In particular, do not
expect `ecci path/to/file` or `ecci path/to/directory` to work at this stage.

## Configuration discovery

For a target file, the internal configuration adapter delegates resolution to
EditorConfig Core C. That means settings are obtained from `.editorconfig`
files applicable to that file, following the normal EditorConfig discovery and
precedence rules, including `root = true`. See the
[EditorConfig specification](https://spec.editorconfig.org/) for those rules.

The current CLI exercises this resolution only for `Cargo.toml`. General
file and directory selection is planned, not implemented.

## Implemented checking libraries

`ecci-editorconfig` and `ecci-checker` are implementation libraries used by
the CLI; they are not separate end-user commands.

The checker library currently has implementations and tests for these
EditorConfig properties:

- `indent_style`
- `indent_size` (for space indentation)
- `end_of_line`
- `trim_trailing_whitespace`
- `insert_final_newline`
- `max_line_length`

The configuration adapter also parses `tab_width` and `charset`. Charset
validation itself is not implemented yet, so it is not listed as a supported
check. Existing test cases document known edge cases and unfinished behavior;
the list above is not a promise of complete EditorConfig-specification
coverage.

## Exit behavior

There is not yet a stable CLI result-reporting or exit-status contract for
configuration violations. Do not use the current command in CI to determine
whether files conform; a future CLI release will define that behavior.

## Development and testing

Run the workspace test suite with:

```sh
cargo test --workspace
```

The tests use fixtures under [`testdata`](testdata) and cover the internal
configuration adapter and checker libraries. Some tests are intentionally
ignored to record known defects. At the current revision, two non-ignored tests
also fail because `unset` is not yet handled for
`insert_final_newline` and `max_line_length`; see the test output before
treating the suite as a release gate.

Before contributing, also run the checks used by CI:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --release
cargo clippy
```

## Roadmap

The next end-user-facing work is:

- CLI directory traversal with `.gitignore` support.
- Binary-file detection.
- A future GitHub Action built on the completed CLI.

These are planned capabilities, not features of the current command.

## License

This project is distributed under the [MIT License](LICENSE).
