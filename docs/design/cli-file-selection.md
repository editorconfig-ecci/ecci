# CLI file selection

## Purpose

This document defines how the `ecci` command-line interface (CLI) turns file
and directory arguments into files that are checked. It is a design contract
for a future CLI implementation; it does not describe the current prototype.

The design keeps project-owned exclusion rules deterministic while preserving
the useful command-line convention that an explicitly named file is checked.
It also separates file selection from result presentation. The output-format
design owns the formats and field names, but must be able to render the
selection outcomes defined here for both people and programs.

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
`.gitignore`. It uses the same pattern syntax, including negated patterns that
re-include a path previously excluded by an applicable `.ecciignore` rule.
`.ecciignore` is the only explicit project-level include/exclude mechanism;
the CLI does not provide `--include` or `--exclude` options.

For a discovered file, selection proceeds in this order:

1. Skip symbolic links.
2. Apply `.gitignore`; a matching exclusion skips the file and prevents
   `.ecciignore` from re-including it.
3. Apply `.ecciignore`; its negated patterns can re-include only paths that
   were excluded by `.ecciignore`.
4. Resolve the file's applicable `.editorconfig` configuration.
5. Skip the file when no `.editorconfig` applies; otherwise, submit it to the
   checker.

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

The selection layer must emit a structured outcome for every file or path it
does not submit to checking. At minimum, outcomes distinguish:

- `gitignore` exclusion;
- `.ecciignore` exclusion;
- no applicable `.editorconfig`;
- symbolic link encountered during traversal;
- direct-file ignore override;
- nonexistent or unsupported direct path;
- broken direct symbolic link; and
- filesystem or permission error.

An outcome includes the affected path and, for errors, enough diagnostic
detail to identify the failing operation. The output layer decides the textual
and machine-readable representation, but must expose these same reason
categories. In particular, it must support a human-readable diagnostic for
operational errors and a machine-readable representation of selection
decisions. Presentation defaults, verbosity controls, and serialization
schemas are outside this design.

### Exit status

The CLI determines its final status after processing every possible argument.

| Status | Meaning |
| --- | --- |
| `0` | No check violations and no operational errors. |
| `1` | One or more check violations, with no usage or operational error. |
| `2` | One or more operational errors. |
| `3` | Invalid CLI syntax or usage. |

When several categories occur, precedence is `3`, then `2`, then `1`, then
`0`. Thus, an unreadable subtree produces status `2` even when another file
also violates its EditorConfig settings.

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
| `.ecciignore` | Root and nested files apply hierarchically; negation re-includes a path excluded by `.ecciignore`; it cannot re-include a Git-ignored path. |
| Direct paths | A direct ignored regular file is selected; a direct directory honors ignore files; missing, unsupported, and broken-link paths are operational errors while later arguments continue. |
| Filesystem traversal | Hidden files are candidates; traversal skips file and directory symlinks; a direct symlink to a regular file is checked; duplicate inputs do not produce duplicate checks. |
| EditorConfig | A file with applicable configuration reaches the checker; a file with no applicable `.editorconfig` is skipped with the correct reason. |
| Reporting | Every skip and error has its required reason category and path; errors contain operation detail for presentation by the output layer. |
| Exit status | Test each individual status and combinations proving `3 > 2 > 1 > 0` precedence. |

## Unresolved related work

The output-format task will decide the supported human-readable and
machine-readable formats, stable field names, verbosity controls, and whether
individual skip outcomes are shown by default. It must preserve the selection
reason categories and error details required by this document. No other file
selection behavior is intentionally left undecided.
