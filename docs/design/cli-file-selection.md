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

When no positional path is supplied, the CLI behaves as if one positional
argument, `.`, had been supplied. The invocation working directory is therefore
the default traversal root. This is a syntactic default, not a direct-file
override: all files discovered below `.` still follow the normal ignore and
binary-selection rules.

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

Traversal never enters a directory named exactly `.git`, at any depth. This is
an unconditional traversal invariant rather than an ignore rule, so neither a
`.gitignore` nor `.ecciignore` negation can re-include it. Naming `.git` or one
of its descendant directories as a traversal root does not bypass the
invariant. A regular file named `.git` remains a candidate, as do paths with
different names such as `.github`.

The implementation must retain enough identity information to check a file at
most once when the same file is supplied by more than one argument. Identity is
the resolved regular file, not the spelling of the input path. On platforms
that expose a stable filesystem object identifier, use that identifier (for
example, device and inode on Unix); otherwise use a canonical absolute path
with the platform's normal path comparison semantics. Consequently, a direct
symbolic link and its target, two direct symbolic links to the same target, and
a direct path also found through traversal produce one checked-file record.

Directness is merged independently of identity. If any occurrence is a direct
file, the merged candidate has direct-file semantics and bypasses ignore
exclusion, even when a traversal occurrence was discovered first. The first
occurrence in argument and traversal order supplies the display path; merging a
later direct occurrence changes selection policy but not display spelling.

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

Ignore matching is a selection-policy API, not an incidental boolean returned
by the directory walker. For every discovered regular-file candidate, that API
must return a structured record containing the final applicable `.gitignore`
match, the final applicable `.ecciignore` match, and the effective decision.
Each recorded match distinguishes no match, ignore, and whitelist, and records
the source ignore-file path and rule location when the matching library exposes
them. The effective record also contains `excluded_by` (`gitignore`,
`ecciignore`, or none) and `force_check`, which is true only for a final
`.ecciignore` whitelist match. This record crosses the boundary from discovery
into selection and reporting; callers must not reconstruct it from whether the
walker yielded a path. Directory pruning may use the same matcher, but must not
replace this per-candidate record.

The walker applies the effective ignore decision to directories and prunes an
excluded directory before reading its contents. Re-inclusion rules for a
descendant therefore follow Git's requirement that excluded parent directories
must also be re-included. Files reached through traversed directories still
retain the per-candidate decision record described above.

For a discovered file, selection proceeds in this order:

1. Skip symbolic links.
2. Apply `.gitignore` and `.ecciignore`; the final applicable `.ecciignore`
   rule takes precedence over `.gitignore`.
3. Skip an excluded file unless a final negated `.ecciignore` rule re-includes
   it.
4. Resolve the file's applicable `.editorconfig` configuration.
5. Skip the file when no `.editorconfig` applies; otherwise, classify its
   content according to [Binary-file detection](binary-file-detection.md).
6. Record the classifier result. Skip a binary file unless the ignore-decision
   record has `force_check=true`; in that case submit it to the checker despite
   the `Binary` result. Submit text files normally.

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

Skip diagnostics are hidden in normal CLI output by default, while every skip
is still retained in the typed report and included in the aggregate skipped
count. `--show-skips` emits one `ECCI-SKIP` warning per retained skip. This flag
changes presentation only; it does not change selection, counts, or exit
status. There is no negative form because hidden is the default.

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
| Defaults and traversal | Omitting positional paths behaves exactly like a single `.` directory argument; hidden files are candidates; traversal skips file and directory symlinks. |
| Git metadata directories | Direct and nested directories named exactly `.git` are never enumerated, including without ignore files and when a negated ignore pattern attempts to re-include them; regular files named `.git` and differently named paths remain candidates. |
| File identity | A target named directly, through one or more direct symlinks, and through traversal is checked once; any direct occurrence gives the merged candidate direct-file semantics, while the first occurrence supplies its display path. |
| EditorConfig and binary detection | A file with applicable configuration reaches binary detection and then the checker; a file with no applicable `.editorconfig` is skipped with the correct reason; binary selection follows the binary-file design. |
| Ignore decision API | Tests cover parent and nested rules, ignore followed by whitelist and whitelist followed by ignore, and disagreement between `.gitignore` and `.ecciignore`; the structured result records both final source matches, the effective exclusion, and `force_check` before binary classification. |
| Reporting | Every skip and error has its required reason category and path; errors contain operation detail for presentation by the output layer; skips are counted but hidden by default and `--show-skips` renders them without changing status or counts. |
| Exit status | Test selection errors through the shared contract: CLI configuration error `2`, I/O error `3`, violation `1`, and intentional skips `0`; execution errors take precedence over violations. |

## Unresolved related work

The [CLI diagnostics and GitHub Action](cli-diagnostics-and-github-action.md)
design defers JSON/SARIF schemas. Those choices
must preserve the selection reason categories and error details required by
this document. No other file-selection behavior is intentionally left
undecided.
