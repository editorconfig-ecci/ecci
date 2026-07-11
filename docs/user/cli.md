# CLI usage

`ecci` selects specified files and directories according to their applicable
`.editorconfig` and project ignore rules. Pass one or more paths, or omit them
to search the current directory:

```sh
cargo run --package ecci -- path/to/file path/to/directory
cargo run --package ecci
```

Directories are searched recursively, including files whose names begin with
`.`. Symbolic links found during traversal are not followed. A symbolic link
named directly is accepted only when it resolves to a regular file. A file is
selected at most once even when multiple arguments refer to the same file.

Directly named regular files bypass `.gitignore` and `.ecciignore`. Directory
contents follow hierarchical ignore rules. See
[`.ecciignore` configuration](ecciignore.md) for syntax, precedence, negation,
and binary force-check behavior.

Files without applicable EditorConfig properties are skipped. Other candidates
are subjected to deterministic binary detection before being selected. The
current command prints one selected path per line; the stable conformance
diagnostic and summary interface remains under development.

## Configuration discovery

For a target file, the configuration adapter delegates resolution to
EditorConfig Core C. Settings are obtained from applicable `.editorconfig`
files using normal EditorConfig discovery and precedence, including
`root = true`. See the [EditorConfig specification](https://spec.editorconfig.org/)
for details.

## Checks

The checker libraries implement checks for these EditorConfig properties:

- `indent_style`
- `indent_size` for space indentation
- `end_of_line`
- `charset`
- `trim_trailing_whitespace`
- `insert_final_newline`
- `max_line_length`

The configuration adapter also parses `tab_width`.

## Exit behavior

Missing paths, broken direct symbolic links, unsupported direct paths, and
filesystem failures are reported as `ECCI-IO` errors. Selection continues for
independent arguments, then exits with status 3 if any such error occurred.
Ordinary exclusions, missing applicable configuration, and binary exclusions
do not cause failure. Stable exit behavior for conformance violations is not
implemented yet.
