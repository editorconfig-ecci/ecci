# CLI file selection

## Purpose

This document defines how the `ecci` command-line interface (CLI) turns file
and directory arguments into files that are checked. It records the contract
implemented by the current CLI.

The design keeps project-owned exclusion rules deterministic while preserving
the useful command-line convention that an explicitly named file is checked.
It integrates with the typed reporting and exit-status contract in
[CLI diagnostics and GitHub Action](cli-diagnostics-and-github-action.md).
That document owns rendered diagnostics, report fields, and final process
status; this document supplies the selection outcomes that it reports.

## Decisions and behavior

### Inputs and traversal

Each positional argument is classified independently.

- A directly specified regular file is a direct file.
- A directly specified directory is a traversal root and is walked
  recursively.
- A directly specified symbolic link is accepted only when it resolves to a
  regular file; it is then handled as a direct file.
- A nonexistent path, a broken direct symbolic link, or a direct path that is
  neither a file nor a directory is an operational error.

Traversal considers regular files, including names beginning with `.`. It does
not follow symbolic links found during traversal, whether they point to files
or directories. Such entries are skipped with the `symlink` reason. This
avoids cycles, duplicate checks, and unexpectedly leaving the requested tree.

The implementation must retain enough identity information to check a file at
most once when the same file is supplied by more than one argument. A direct
file remains direct for this purpose even if it was already discovered through
a directory argument.

### Ignore rules and selection order

For every traversal root, `.gitignore` files are discovered from that root
downward and applied hierarchically using Git ignore-pattern syntax and
precedence. Rules in a deeper `.gitignore` apply relative to their containing
directory and take precedence over applicable parent rules. Only `.gitignore`
files participate: `.git/info/exclude` and the user's global Git excludes file
are deliberately not read. Therefore, the selection result does not vary with
per-clone or per-user Git configuration.

`.ecciignore` is discovered and applied hierarchically in the same way as
`.gitignore`. It uses the same pattern syntax, including negated patterns. Its
rules have higher precedence than `.gitignore` rules. A final negated
`.ecciignore` match is a force-check rule: it re-includes a regular file and
also prevents binary-file exclusion. The `.ecciignore` file itself is never
checked. `.ecciignore` is the only explicit project-level include/exclude
mechanism; the CLI does not provide `--include` or `--exclude` options.

For a discovered file, selection proceeds in this order:

1. Skip symbolic links.
2. Apply `.gitignore` and `.ecciignore`; the final applicable `.ecciignore`
   rule takes precedence over `.gitignore`.
3. Skip an excluded file unless a final negated `.ecciignore` rule re-includes
   it.
4. Resolve the file's applicable `.editorconfig` configuration.
5. Skip the file when no `.editorconfig` applies; otherwise, classify its
   content according to [Binary-file detection](binary-file-detection.md).
6. Skip a binary file unless a final negated `.ecciignore` rule force-selects
   it; submit the remaining file to the checker.

A direct file bypasses both ignore mechanisms and is submitted to
EditorConfig resolution even if `.gitignore` or `.ecciignore` would exclude
it. This forced selection does not turn a directly specified directory into a
forced traversal: files found below a directory still follow the sequence
above.

There is no implicit include filter. Files not excluded by either ignore
mechanism remain candidates, subject to EditorConfig resolution.

### Errors, skips, and reporting

Failure to enumerate a directory, inspect an entry, read a needed ignore file,
or read a candidate file is an operational error. The CLI continues with other
independent entries and arguments after recording an operational error. A
missing applicable `.editorconfig` is not an error: it is an intentional skip.

The selection layer must retain a structured outcome for every file or path it
does not submit to checking. At minimum, outcomes distinguish:

- `gitignore` exclusion;
- `.ecciignore` exclusion;
- no applicable `.editorconfig`;
- binary-file exclusion;
- symbolic link encountered during traversal;
- direct-file ignore override;
- nonexistent or unsupported direct path;
- broken direct symbolic link; and
- filesystem or permission error.

An outcome includes the affected path and, for errors, enough diagnostic
detail to identify the failing operation. The reporting contract maps
intentional skips to `ECCI-SKIP` diagnostics when skips are shown and maps
operational errors to `ECCI-IO`. It defines the human-readable text format,
summary, and future machine-readable formats.

### Exit status

The selection layer records its outcomes and does not assign an aggregate exit
status independently. The shared [exit-status contract](cli-diagnostics-and-github-action.md#exit-status-contract)
applies: an invalid CLI option is a configuration error (`2`), while a missing
explicit path or traversal/read failure is an I/O error (`3`). Either execution
error takes precedence over check violations (`1`), and intentional skips do
not fail an otherwise successful invocation (`0`).

## Alternatives considered

| Decision | Chosen approach | Rejected alternative and rationale |
| --- | --- | --- |
| Git exclusions | Hierarchical `.gitignore` only | Reading `.git/info/exclude` and global excludes would make results depend on clone-local and user-local state. |
| Hidden files | Include by default | Excluding dotfiles would omit common project files, including configuration and CI files. |
| Symbolic links | Do not follow traversal links | Following links requires cycle detection and can check files outside the requested tree. |
| Explicit filters | Hierarchical `.ecciignore` | CLI `--include`/`--exclude` options duplicate persistent project policy and make invocation-specific behavior harder to reproduce. |
| Direct ignored files | Force-select direct files | Users expect a file they name explicitly to be checked. |
| Missing configuration | Skip | Files without an applicable `.editorconfig` have no settings against which to check. |
| Error handling | Continue and return an operational-error status | Failing fast hides independent violations and prevents a complete report. |

## Acceptance criteria and test matrix

An implementation conforms to this design when the following cases are
covered by automated CLI or selection-layer tests.

| Area | Scenarios |
| --- | --- |
| `.gitignore` | Root and nested files apply relative patterns; a nested rule overrides a parent rule; `.git/info/exclude` and global excludes do not affect results. |
| `.ecciignore` | Root and nested files apply hierarchically; its rules take precedence over `.gitignore`; a final negated rule re-includes a regular file and force-selects it for binary detection. |
| Direct paths | A direct ignored regular file is selected; a direct directory honors ignore files; missing, unsupported, and broken-link paths are operational errors while later arguments continue. |
| Filesystem traversal | Hidden files are candidates; traversal skips file and directory symlinks; a direct symlink to a regular file is checked; duplicate inputs do not produce duplicate checks. |
| EditorConfig and binary detection | A file with applicable configuration reaches binary detection and then the checker; a file with no applicable `.editorconfig` is skipped with the correct reason; binary selection follows the binary-file design. |
| Reporting | Every skip and error has its required reason category and path; errors contain operation detail for presentation by the output layer. |
| Exit status | Test selection errors through the shared contract: CLI configuration error `2`, I/O error `3`, violation `1`, and intentional skips `0`; execution errors take precedence over violations. |

## Unresolved related work

The [CLI diagnostics and GitHub Action](cli-diagnostics-and-github-action.md)
design defers JSON/SARIF schemas and skip-presentation defaults. Those choices
must preserve the selection reason categories and error details required by
this document. No other file-selection behavior is intentionally left
undecided.
