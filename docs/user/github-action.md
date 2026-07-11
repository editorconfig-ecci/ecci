# GitHub Action

The ecci GitHub Action checks repository files against their applicable
`.editorconfig` settings. It uses workflow commands for annotations and the
job summary, so it does not call the GitHub API or require a write token.

## Basic usage

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: editorconfig-ecci/ecci@v1
    with:
      paths: |
        src
        tests
```

Each non-empty line in `paths` is a repository-relative file or directory.
Directories are searched recursively. Paths and `working-directory` must
resolve inside `GITHUB_WORKSPACE`; absolute paths and escapes through `..` or
symbolic links are rejected.

## Inputs

| Input | Default | Description |
| --- | --- | --- |
| `paths` | `.` | Newline-separated repository-relative files or directories. Empty lines are invalid. |
| `working-directory` | `.` | Directory inside the workspace from which paths are resolved. |
| `fail-on-violation` | `true` | When `false`, report violations without failing the step. Execution errors still fail. |
| `annotations` | `true` | Emit location-aware workflow annotations. |
| `summary` | `true` | Append one bounded result section to the job summary. |
| `max-annotations` | `50` | Maximum annotations. `0` disables them; suppressed diagnostics are counted. |
| `log-level` | `summary` | `quiet`, `summary`, `diagnostic`, or `debug`. |

Boolean values are case-sensitive and accept only `true` or `false`.
`max-annotations` accepts only a non-negative decimal integer. Invalid inputs
are configuration errors.

## Outputs and results

| Output | Description |
| --- | --- |
| `outcome` | `success`, `violations`, `configuration-error`, `io-error`, or `internal-error`. |
| `violations` | Number of violations. |
| `checked-files` | Number of files fully checked. |
| `skipped-files` | Number of intentionally skipped files. |

With `fail-on-violation: false`, an `outcome` of `violations` has a successful
step status. Configuration, input/output, and internal errors always fail. The
Action's annotations appear on the workflow check when GitHub supports the
reported location; no pull-request comment or additional permission is used.
