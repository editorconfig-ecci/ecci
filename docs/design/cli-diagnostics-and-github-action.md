# CLI diagnostics and GitHub Action

## Status and scope

This document defines the public result-reporting contract for the first stable
`ecci` command-line interface (CLI) and its GitHub Action. The shared in-memory
model and text renderer are implemented in `ecci-report`; wiring the model into
target selection, checker execution, the CLI, and the Action remains separate
work. The examples below therefore define required behavior and are not a
claim that the current CLI prototype exposes it yet.

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
available. Paths are relative to the invocation working directory whenever
possible and use forward slashes in rendered output.

The initial text format is intended for terminals and CI logs, not as a
machine-readable API. One diagnostic is rendered on one line; the location is
omitted when unavailable. Findings use `error`, skipped work uses `warning`,
and fatal execution errors use `error`. Detailed progress is never mixed into
the diagnostic stream unless requested.

```text
error[ECCI001] src/lib.rs:14:1: indent_style must be space; found tab
error[ECCI002] src/lib.rs:47:81: max_line_length must be 80; found 96
warning[ECCI-SKIP] docs/generated.md: no .editorconfig applies; skipped
Checked 2 files: 2 violations, 1 skipped, 0 execution errors.
```

The category codes are stable identifiers for documentation, JSON, SARIF, and
tests. `ECCI001` and similar property-check codes identify violations;
`ECCI-CONFIG`, `ECCI-IO`, and `ECCI-INTERNAL` identify the execution-error
categories; `ECCI-SKIP` identifies an intentional skip. The message may gain
context but must not be the only way to classify a diagnostic.

The following cases have these required meanings and text examples:

| Case | Required diagnostic and summary behavior | Example |
| --- | --- | --- |
| Violation | Emit one `error[ECCIxxx]` finding for each reportable violation, retain checking other files, and summarize the count. | `error[ECCI001] src/lib.rs:14:1: indent_style must be space; found tab` |
| Configuration error | Emit `error[ECCI-CONFIG]`; do not claim the affected target was checked. Continue only with independent targets when safe. | `error[ECCI-CONFIG] .editorconfig:12: invalid indent_size value "many"` |
| I/O error | Emit `error[ECCI-IO]` with the affected path and operation; do not claim that target was checked. Continue only with independent targets when safe. | `error[ECCI-IO] src/lib.rs: failed to read file: Permission denied` |
| Internal error | Emit exactly one user-safe `error[ECCI-INTERNAL]` summary, without a backtrace by default, and stop normal checking. | `error[ECCI-INTERNAL]: unexpected checker failure; rerun with --debug and report this error` |
| No targets | Emit no per-file diagnostic and a summary that explicitly says no targets were selected. | `Checked 0 files: no targets selected.` |
| No applicable `.editorconfig` | Treat the target as skipped, not conforming or failing; emit a warning only when skips are shown and count it in the summary. | `warning[ECCI-SKIP] README.md: no .editorconfig applies; skipped` |

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
report model as the CLI. Its first-release interface is proposed as follows:

| Input | Required | Default | Meaning |
| --- | --- | --- | --- |
| `paths` | No | `.` | Newline-separated repository-relative files or directories to check. |
| `working-directory` | No | `${{ github.workspace }}` | Directory used to resolve `paths` and relative locations. It must remain inside the workspace. |
| `fail-on-violation` | No | `true` | When `true`, violations make the Action fail; when `false`, they are reported but the Action succeeds unless an execution error occurs. |
| `annotations` | No | `true` | Emit GitHub Actions annotations when running in GitHub Actions. |
| `summary` | No | `true` | Write the aggregate result to the GitHub Actions job summary. |
| `max-annotations` | No | `50` | Maximum file-level annotations. Excess findings remain counted and are listed compactly in the summary. `0` disables annotations. |
| `log-level` | No | `summary` | One of `quiet`, `summary`, `diagnostic`, or `debug`; controls ordinary log volume, never the final status. |

Boolean and numeric inputs must be validated strictly. Invalid input is an
Action configuration error, produces an `ECCI-CONFIG` annotation when possible,
and fails the Action. The Action must reject a `working-directory` or path that
resolves outside `GITHUB_WORKSPACE`.

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
are applied independently of log level. The Action must report suppression
counts rather than silently dropping output.

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
- Which target-selection and ignore-file semantics will the CLI expose, and how
  will they distinguish an empty selection from an excluded path? This design
  fixes their result semantics but not their selection syntax.
- Should the future JSON format be one complete document, newline-delimited
  JSON records, or support both? The choice affects streaming and schema
  versioning.
- Is SARIF export sufficient for users who need GitHub code scanning, or should
  the Action later offer an opt-in upload mode? Uploading would introduce token
  permissions and a different failure model.
