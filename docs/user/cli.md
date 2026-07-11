# CLI usage

`ecci` is intended to check specified files and directories against their
applicable `.editorconfig` settings. The command-line interface is still under
development: it currently has no options or path arguments, does not traverse
directories, and does not report check results.

Run the current prototype from the repository root:

```sh
cargo run --package ecci
```

It reads the repository-root `Cargo.toml`, resolves its EditorConfig settings,
and prints its `indent_style`. With this repository's current configuration,
the output is:

```text
Cargo.toml indent_style:Space
```

Do not expect `ecci path/to/file` or `ecci path/to/directory` to work yet.

## Configuration discovery

For a target file, the internal configuration adapter delegates resolution to
EditorConfig Core C. Settings are obtained from the `.editorconfig` files that
apply to that file, following normal EditorConfig discovery and precedence
rules, including `root = true`. See the
[EditorConfig specification](https://spec.editorconfig.org/) for details.

The current CLI exercises this resolution only for `Cargo.toml`. General file
and directory selection is planned, not implemented.

## Checks

`ecci-editorconfig` and `ecci-checker` are implementation libraries, not
separate end-user commands. The checker library currently has implementations
and tests for these EditorConfig properties:

- `indent_style`
- `indent_size` for space indentation
- `end_of_line`
- `charset`
- `trim_trailing_whitespace`
- `insert_final_newline`
- `max_line_length`

The configuration adapter also parses `tab_width`. Existing tests record known
edge cases and unfinished behavior; the implemented list does not claim
complete EditorConfig-specification coverage.

## Exit behavior

There is no stable CLI result-reporting or exit-status contract for
configuration violations. Do not use the current command in continuous
integration to determine whether files conform.

The underlying typed report model and human-oriented text renderer now
implement the agreed diagnostic categories, aggregate summary, and exit-status
calculation. They are not connected to the current CLI prototype yet. Once the
checker execution path is connected, diagnostics will be human-facing output,
not a machine-readable interface; integrations such as the GitHub Action will
consume the typed report directly instead of parsing CLI text.

## Roadmap

The planned end-user-facing work is:

- CLI directory traversal with `.gitignore` support.
- Binary-file detection.
- A future GitHub Action built on the completed CLI.

These are planned capabilities, not current CLI features.
