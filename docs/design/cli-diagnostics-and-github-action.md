# CLI diagnostics and GitHub Action

## Status and scope

This document defines the public result-reporting contract for the `ecci`
command-line interface (CLI) and its GitHub Action. The shared model, text
renderer, target selection, checker-backed CLI, and Docker Action presentation
are implemented.

The design distinguishes a **finding** (a checked file does not conform) from
an **execution error** (the command could not reliably complete the requested
work). This distinction is reflected in both diagnostics and exit status.

## Output-format decision

| Format | Human readability | Automation support | GitHub pull-request integration | Implementation and maintenance cost | Decision |
| --- | --- | --- | --- | --- | --- |
| Human-oriented text | High | Limited; text is not a stable parsing API | Useful in job logs | Low | Required CLI format for the initial release |
| JSON | Low without tooling | High; suitable for scripts and IDE integrations | Requires a consumer to create annotations | Medium; requires a versioned schema | Deferred extension |
| GitHub Actions annotations | High in the Checks UI | Specific to GitHub Actions | Native file and line annotations | Low in the Action, but not portable | Required Action presentation in the initial release |
| SARIF (Static Analysis Results Interchange Format) | Low without a viewer | High for code-scanning tools | Native code-scanning upload, where supported | Higher; requires a precise rule and location mapping | Deferred extension |

The initial release therefore has two required presentations:

- The CLI emits concise, human-oriented text diagnostics and a summary.
- The GitHub Action emits GitHub Actions annotations for reportable findings
  and execution errors, plus a job summary.

The CLI must build a typed in-memory report before rendering either
presentation. Renderers must not parse the text renderer's output. This keeps
the text format free to improve for people while making a future JSON renderer
and SARIF exporter reliable. JSON, when introduced, must be an explicitly
selected, versioned format (for example, `--format json` with a top-level
`schema_version`); it is not implied by the text format. SARIF is an export of
the same report model and must use stable rule identifiers rather than treating
individual messages as identifiers.

The `ecci-report` crate owns this presentation-independent model. It represents
findings, intentional skips, the three execution-error categories, optional
paths and one-based locations, property and expected/observed values, stable
codes, counters, and the invocation exit status. CLI text, Action annotations,
job summaries, and future structured formats consume the same typed report.
Counters are derived from report entries so renderers cannot disagree about
aggregate results.

## Diagnostic model and text output

Each diagnostic has a severity, category, message, optional path, and optional
one-based line and column. A finding additionally identifies the checked
EditorConfig property, expected value, and observed value when those values are
available. CLI paths use the invocation working directory captured at process
startup as their display base. The first selected spelling is made absolute
against that base and lexically normalized, but is not canonicalized merely for
display; it is then rendered relative to the base. This preserves a user's
direct symbolic-link spelling. If a relative path cannot be represented, such
as across Windows volumes, the normalized absolute spelling is rendered. Paths
use forward slashes in all rendered output. Display-path choice never supplies
file identity or workspace-containment evidence.

The initial text format is intended for terminals and CI logs, not as a
machine-readable API. One diagnostic is rendered on one line; the location is
omitted when unavailable. Findings use `error`, skipped work uses `warning`,
and fatal execution errors use `error`. Detailed progress is never mixed into
the diagnostic stream unless requested. Skip diagnostics are not shown by
default. `--show-skips` requests their `selection.skipped` warning lines; skipped files
remain counted in either mode.

```text
error[indent_style.invalid_value] src/lib.rs:14:1: indent_style must be space; found tab
error[max_line_length.exceeded] src/lib.rs:47:81: max_line_length must be 80; found 96
warning[selection.skipped] docs/generated.md: no .editorconfig applies; skipped
Checked 2 files: 2 violations, 1 skipped, 0 execution errors.
```

Codes are stable identifiers for documentation, JSON, SARIF, and tests. A
property finding uses `<property>.<kind>`. The checker, which knows the precise
failure, supplies the complete code rather than relying on a renderer-side
property-to-code table. A new check for an existing property must receive a
distinct kind when users can act on it differently. Kinds use lower-case
snake_case and describe the observed failure, such as `invalid_value`,
`missing`, `present`, or `exceeded`; an existing code must never be repurposed.

Diagnostics without a direct EditorConfig property use reserved namespaces:
`config` for command-line, Action-input, syntax, and configuration-resolution
failures; `io` for filesystem operations; `internal` for invariant failures;
and `selection` for target-selection outcomes. Their form is likewise
`<namespace>.<kind>`. Current category-level codes are `config.invalid`,
`io.failed`, `internal.unexpected`, and `selection.skipped`. More specific kinds
may be added without colliding with property names. The message may gain
context but must not be the only way to classify a diagnostic.

This migration is intentionally breaking. The project is still on a `0.x`
release line, and the release workflow classifies such changes with its
`breaking`/`major` category. Emitting both identifiers would make the code field
ambiguous, so renderers expose only the new identifier. The complete migration
inventory is:

| Previous code | Stable string code | Producer |
| --- | --- | --- |
| `ECCI001` | `indent_style.invalid_value` | `indent_style` checker |
| `ECCI002` | `indent_size.invalid_value` | `indent_size` checker |
| `ECCI003` | `end_of_line.invalid_value` | `end_of_line` checker |
| `ECCI004` | `charset.invalid_value` | `charset` checker |
| `ECCI005` | `trim_trailing_whitespace.present` | `trim_trailing_whitespace` checker |
| `ECCI006` | `insert_final_newline.missing` | `insert_final_newline` checker |
| `ECCI007` | `max_line_length.exceeded` | `max_line_length` checker |
| `ECCI-CONFIG` | `config.invalid` | CLI, configuration resolver, and Action input validation |
| `ECCI-IO` | `io.failed` | file selection and checking operations |
| `ECCI-INTERNAL` | `internal.unexpected` | report and CLI failure boundary |
| `ECCI-SKIP` | `selection.skipped` | file selection |

The text renderer preserves `severity[code] location: message`. GitHub Action
annotation titles and job-summary finding entries use the same code from the
typed report. Tests cover the report model and renderer, CLI output, Action log
output, and container-entrypoint validation.

The following cases have these required meanings and text examples:

| Case | Required diagnostic and summary behavior | Example |
| --- | --- | --- |
| Violation | Emit one property finding for each reportable violation, retain checking other files, and summarize the count. | `error[indent_style.invalid_value] src/lib.rs:14:1: indent_style must be space; found tab` |
| Configuration error | Emit `error[config.invalid]`; do not claim the affected target was checked. Continue only with independent targets when safe. | `error[config.invalid] .editorconfig:12: invalid indent_size value "many"` |
| I/O error | Emit `error[io.failed]` with the affected path and operation; do not claim that target was checked. Continue only with independent targets when safe. | `error[io.failed] src/lib.rs: failed to read file: Permission denied` |
| Internal error | Emit exactly one user-safe `error[internal.unexpected]` summary, without a backtrace by default, and stop normal checking. | `error[internal.unexpected]: unexpected checker failure; rerun with --debug and report this error` |
| No targets | Emit no per-file diagnostic and a summary that explicitly says no targets were selected. | `Checked 0 files: no targets selected.` |
| No applicable `.editorconfig` | Treat the target as skipped, not conforming or failing; emit a warning only when skips are shown and count it in the summary. | `warning[selection.skipped] README.md: no .editorconfig applies; skipped` |

`--debug` may add causal-chain details for execution errors to stderr, but it
must not change the category, exit code, or the normal diagnostic message.
The model accepts debug causes only through the explicitly named
`SafeDebugDetail::from_sanitized` boundary. Producers must remove secrets,
environment values, target-file content, backtraces, and host-specific absolute
paths before crossing that boundary. Normal rendering never reads these
details; debug rendering opts in explicitly and keeps every cause on one line.

## Exit-status contract

The process exits with one status for the entire invocation. An execution error
takes precedence over findings because a partial check must not be reported as
a reliable conformance result. If several execution-error categories occur,
use the highest-precedence status in this order: internal error, configuration
error, then I/O error.

| Exit code | Meaning | Conditions |
| --- | --- | --- |
| 0 | Completed without violations | All checked files conform, or no targets were selected, or all selected targets were intentionally skipped because no applicable `.editorconfig` was found. |
| 1 | Violations found | At least one finding was emitted and no execution error occurred. |
| 2 | Configuration error | An `.editorconfig` file or supplied checker configuration could not be read, parsed, or interpreted reliably. |
| 3 | I/O error | A required target, directory traversal operation, or output operation failed. |
| 4 | Internal error | An invariant failure, panic converted at the CLI boundary, or another unexpected `ecci` failure occurred. |

An unsupported command-line option or invalid option value is a configuration
error and exits 2. A nonexistent explicitly named path is an I/O error and
exits 3. Selecting no paths after normal selection and ignore rules is not an
error; it exits 0 so that repository-wide CI does not fail merely because a
filter matched nothing.

## GitHub Action interface

The published Action is a Docker Action that invokes the same checker and
report model as the CLI. Its first-release interface is defined as follows:

| Input | Required | Default | Meaning |
| --- | --- | --- | --- |
| `paths` | No | `.` | Newline-separated files or directories, relative to `working-directory`, to check. |
| `working-directory` | No | `.` | Workspace-relative directory used to resolve `paths`. It must resolve to an existing directory inside the workspace. |
| `fail-on-violation` | No | `true` | When `true`, violations make the Action fail; when `false`, they are reported but the Action succeeds unless an execution error occurs. |
| `annotations` | No | `true` | Emit GitHub Actions annotations when running in GitHub Actions. |
| `summary` | No | `true` | Write the aggregate result to the GitHub Actions job summary. |
| `max-annotations` | No | `50` | Maximum file-level annotations. Excess findings remain counted and are listed compactly in the summary. `0` disables annotations. |
| `log-level` | No | `summary` | One of `quiet`, `summary`, `diagnostic`, or `debug`; controls ordinary log volume, never the final status. |

Boolean and numeric inputs must be validated strictly. Invalid input is an
Action configuration error, produces a `config.invalid` annotation when possible,
and fails the Action. The Action must reject a `working-directory` or path that
resolves outside `GITHUB_WORKSPACE`.

`GITHUB_WORKSPACE` is the immutable containment and annotation base. The Action
canonicalizes it first. `working-directory` must be relative (an absolute value
is rejected), is resolved against that base, and is lexically normalized. Both
that lexical result and its canonical result must be contained by the canonical
workspace. It must name an existing directory. Thus a lexical escape is not
made valid by a symbolic link back into the workspace, and a contained lexical
path is rejected if symbolic-link resolution escapes. The default `.` means the
workspace root.

The Action parses `paths` by splitting on lines, removing a trailing carriage
return, trimming leading and trailing ASCII whitespace from each line, and
discarding empty lines. GitHub Actions does not reliably distinguish an omitted
optional string input from an explicitly empty one, so zero entries after this
processing means one entry, `.`. Paths containing leading or trailing ASCII
whitespace cannot be expressed through this input. Each entry must be relative;
absolute paths are configuration errors.

Each path is resolved against the canonical working directory and lexically
normalized before filesystem access. A lexical `..` is allowed only when the
normalized absolute result remains within the canonical workspace. For an
existing path, the whole path is canonicalized. For a nonexistent path, the
Action canonicalizes its longest existing ancestor and lexically appends the
remaining components. The resulting resolved path must also remain within the
canonical workspace. This catches an escape through an existing symbolic-link
ancestor even when the final component does not exist. A contained nonexistent
path proceeds to normal selection and becomes `io.failed`; either a lexical or
resolved escape is `config.invalid`.

Action input de-duplication uses the lexically normalized absolute path as its
key, with the platform's normal path-comparison semantics, and retains the
first entry. It intentionally does not use canonical identity: the selection
layer performs resolved-file identity de-duplication, so aliases and direct
symbolic links that survive containment are also checked only once while the
first input spelling remains available for display.

CLI text paths are relative to the invocation working directory when possible,
as specified above. In the Action, all log, summary, and annotation paths are
instead relative to the canonical workspace, regardless of
`working-directory`, because workflow annotations use repository-relative
locations. Rendered paths use forward slashes. A path for which a contained,
workspace-relative spelling cannot be produced is omitted from an annotation
rather than rendered as an absolute host path; containment validation normally
prevents this case.

The Action has these outputs:

| Output | Meaning |
| --- | --- |
| `outcome` | `success`, `violations`, `configuration-error`, `io-error`, or `internal-error`. |
| `violations` | Decimal count of reported violations. |
| `checked-files` | Decimal count of files fully checked. |
| `skipped-files` | Decimal count of intentional skips. |

The Action maps findings to `::error file=...,line=...,col=...::` annotations
when a usable repository-relative location exists. A finding without a line or
column is still annotated at file scope. Configuration and I/O errors are also
annotated at their relevant file when possible; an internal error is emitted as
a single workflow-level error annotation. Annotation messages use the stable
code and a compact explanation. The Action must escape GitHub workflow-command
data correctly and must not let target-file content inject commands.

For pull requests, annotations are attached to the workflow check run and
therefore appear in the pull-request Checks experience when GitHub permits it.
The Action must not call the GitHub API, post review comments, or require a
write token. This makes annotation behavior work with the default
`GITHUB_TOKEN` permissions and with fork pull requests, subject to GitHub's
normal annotation display rules.

The job summary is written once, after checking, and includes outcome, checked,
skipped, violation, and execution-error counts; it lists the first limited set
of findings and says how many were omitted. It must be useful even when
`log-level=quiet`. A representative summary is:

```markdown
## ecci

**Outcome:** violations found

| Checked | Violations | Skipped | Execution errors |
| ---: | ---: | ---: | ---: |
| 24 | 3 | 2 | 0 |

Showing 3 of 3 violations. See annotations and logs for details.
```

With the default `fail-on-violation=true`, Action failure conditions are exit
code 1 (violations) and every execution error (2--4). With
`fail-on-violation=false`, a CLI violation result is converted to a successful
Action result with `outcome=violations`; execution errors always fail. This is
the only Action-specific remapping of the CLI exit contract.

`log-level=quiet` writes no individual findings to ordinary logs; `summary`
writes only the final summary; `diagnostic` writes all rendered diagnostics;
and `debug` also writes safe diagnostic context. Annotation and summary limits
are applied independently of log level. For Action log rendering,
`diagnostic` and `debug` enable the CLI's `--show-skips` presentation, while
`quiet` and `summary` hide individual skip warnings; all levels preserve the
skipped count. The Action must report suppression counts rather than silently
dropping output.

## Acceptance criteria

- The implemented CLI returns the specified exit code for every case in the
  exit-status table, including mixed findings and execution errors.
- A violation has a stable code, path, and location whenever the checker can
  determine them; normal text output renders one diagnostic per line and a
  final aggregate summary.
- A target with no applicable `.editorconfig` is counted as skipped and cannot
  by itself fail the CLI or Action.
- The Action validates every input, confines selected paths to the workspace,
  produces the declared outputs, and preserves execution-error failures even
  when `fail-on-violation` is false.
- With no CLI positional path, selection is identical to selecting `.`; skip
  warnings are hidden by default and `--show-skips` changes only their
  presentation.
- Action `diagnostic` and `debug` logs show skip diagnostics, while `quiet` and
  `summary` logs hide them; every log level reports the same skipped count.
- Action path parsing ignores blank lines, treats an empty parsed value as `.`,
  removes normalized duplicates in first-occurrence order, rejects absolute
  paths and workspace escapes, and reports contained nonexistent paths as I/O
  errors.
- CLI display tests prove that paths are based on the startup working
  directory, use forward slashes, preserve the first direct symbolic-link
  spelling, and fall back to a normalized absolute path only when a relative
  spelling cannot be represented.
- Action tests cover a nested working directory, `..` that remains contained,
  lexical escapes, symlink escapes, a nonexistent leaf below a symlink escape,
  duplicate normalized spellings, and workspace-relative forward-slash paths
  in logs, summaries, and annotations.
- On GitHub Actions, the Action emits correctly escaped, location-aware
  annotations up to `max-annotations`, reports the suppressed count, and writes
  one bounded job summary when enabled.
- The Action does not require GitHub API access or write-token permissions to
  provide pull-request annotations.
- JSON and SARIF are not advertised as implemented until their schemas and
  compatibility policies are documented and tested.

## Test strategy

Unit tests should construct typed reports for each category and verify status
precedence, counter aggregation, stable codes, and text rendering with and
without locations. CLI integration tests should use temporary directories to
cover conforming files, violations, malformed `.editorconfig` files,
permission/read failures where portable, nonexistent explicit paths, no target
selection, and no applicable `.editorconfig` skips. They should assert stdout,
stderr where relevant, and the process status without depending on platform
error wording.

Action tests should run the container entrypoint with a simulated workspace and
assert argument construction, strict input validation, workspace confinement,
output-file values, annotation command escaping, annotation limits, summary
limits, and `fail-on-violation` remapping. A GitHub Actions workflow fixture
should verify that the published metadata accepts the documented inputs and
that a violation produces an annotation command without requiring a token.

When JSON or SARIF is added, fixture-based schema validation and compatibility
tests must be added before declaring either output stable.

## Open questions

- Should an explicit policy input make a missing applicable `.editorconfig`
  fail instead of skip? The initial release intentionally skips, matching the
  current discovery-oriented model; a future opt-in policy would need a new
  diagnostic code and documentation.
- Should the future JSON format be one complete document, newline-delimited
  JSON records, or support both? The choice affects streaming and schema
  versioning.
- Is SARIF export sufficient for users who need GitHub code scanning, or should
  the Action later offer an opt-in upload mode? Uploading would introduce token
  permissions and a different failure model.
