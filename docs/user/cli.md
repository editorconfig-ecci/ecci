# CLI usage

`ecci` checks files and directories against their applicable `.editorconfig`
settings. Pass any number of paths; with no path, `ecci` selects `.`.

```sh
ecci [--show-skips] [--debug] [--] [PATH ...]
```

Directories are searched recursively, including dotfiles. Traversal does not
follow symbolic links; a directly named symbolic link is accepted only when it
resolves to a regular file. Duplicate files are checked once. Directly named
files bypass ignore rules, while directory contents follow hierarchical
`.gitignore` and `.ecciignore` rules. See [`.ecciignore`
configuration](ecciignore.md) for precedence, negation, and binary force-check
behavior.

Files without applicable EditorConfig properties and binary files are skipped.
Selection and checking continue after errors that do not prevent independent
targets from being processed.

## Options

- `--show-skips` writes an `ECCI-SKIP` warning for each skipped path. Skips are
  always counted but hidden by default.
- `--debug` adds sanitized causal details for execution errors without changing
  categories, counts, or exit status.
- `--` ends option processing, allowing a path beginning with `-`.

Options accept no value and may appear only once. Unsupported options and
duplicate controls are configuration errors.

## Diagnostics and output

Diagnostics are written to standard error, one per line. Stable codes identify
their categories, and locations use working-directory-relative paths and
one-based lines and columns whenever available. The final summary is written to
standard output.

```text
error[ECCI001] src/lib.rs:14:1: indent_style must be space; found tab
Checked 1 files: 1 violations, 0 skipped, 0 execution errors.
```

Finding codes identify checked properties. `ECCI-CONFIG`, `ECCI-IO`, and
`ECCI-INTERNAL` identify execution errors; `ECCI-SKIP` identifies an intentional
skip. Human-readable text is not a machine-readable API. An empty selection is
reported as `Checked 0 files: no targets selected.`

## Exit status

| Status | Meaning |
| ---: | --- |
| `0` | Files conform, no targets were selected, or all targets were skipped. |
| `1` | Violations were found and no execution error occurred. |
| `2` | A CLI or `.editorconfig` configuration error occurred. |
| `3` | A target, traversal, or file-reading input/output error occurred. |
| `4` | An unexpected internal failure occurred. |

Execution errors take precedence over violations. Multiple execution errors use
internal, configuration, then input/output precedence.

## Configuration and checks

Configuration resolution uses EditorConfig Core C and normal `.editorconfig`
discovery and precedence, including `root = true`. The checker supports
`indent_style`, space `indent_size`, `end_of_line`, `charset`,
`trim_trailing_whitespace`, `insert_final_newline`, and `max_line_length`. The
adapter also parses `tab_width`. See the
[EditorConfig specification](https://spec.editorconfig.org/) for details.
