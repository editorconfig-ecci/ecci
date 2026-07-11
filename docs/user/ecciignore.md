# `.ecciignore` configuration

The `ecci` command-line interface discovers `.ecciignore` files while walking
a directory. Place one at a traversal root or in any directory below it. A
nested file applies to that directory and its descendants. A directly named
file bypasses ignore rules, but files discovered below a directly named
directory do not.

`.ecciignore` uses Git ignore-pattern syntax. Blank lines and lines beginning
with `#` are ignored; `/` anchors a pattern to the directory containing the
ignore file; a trailing `/` matches directories; and `*`, `?`, and `**` provide
the usual Git-style wildcards. Excluded directories are not traversed, so a
negation below an excluded directory must first re-include its parent directory,
as with Git. For example:

```gitignore
# Generated files below this directory
generated/
*.snapshot

# Keep this particular snapshot
!fixtures/reference.snapshot
```

## Precedence and negation

`ecci` also reads hierarchical `.gitignore` files below each traversal root.
It deliberately does not read `.git/info/exclude` or global Git exclusions, so
selection does not depend on clone-local or user-local Git configuration.

Rules in a deeper ignore file take precedence over applicable rules in a
parent directory. The final applicable `.ecciignore` rule takes precedence
over `.gitignore`. A normal `.ecciignore` match excludes the file. A final
negated pattern, written with `!`, re-includes it even when `.gitignore` would
exclude it. The `.ecciignore` file itself is never checked.

## Binary files and force-check rules

Before checking a candidate, `ecci` examines at most its first 8 KiB. Empty
files, files with a UTF-16 byte-order mark, and files configured as `utf-16le`
or `utf-16be` are treated as text. Otherwise, a NUL byte or a disallowed C0
control byte marks the candidate as binary. Tabs, line feeds, carriage returns,
and form feeds are allowed in text.

A final negated `.ecciignore` match is also a **force-check** rule. It bypasses
binary exclusion, which provides an escape hatch for text formats that the
deterministic classifier mistakes for binary. Force-check does not make a
directory, symbolic link, special file, or unreadable file checkable, and it
does not suppress EditorConfig or file-read errors.
