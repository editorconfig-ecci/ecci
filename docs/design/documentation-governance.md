# Documentation governance

## Scope and language

This policy governs repository documentation. All documentation, including new
pages and edits to existing pages, must be written in English.

Documentation is written for people. Instructions for coding agents belong in
the root [AGENTS.md](../../AGENTS.md), with extended agent-only guidance under
`docs/agents/` when it becomes necessary. Do not place agent instructions in
human documentation.

## Structure and audiences

| Location | Audience | Purpose |
| --- | --- | --- |
| `AGENTS.md` | Coding agents | Concise operational instructions and links to extended agent guidance. |
| `docs/agents/` | Coding agents | Extended agent-only procedures, if needed. |
| `README.md` | Project visitors | Brief purpose, installation, basic usage, and links. |
| `docs/user/` | Users | CLI usage, configuration, diagnostics, and GitHub Action usage. |
| `docs/design/` | Developers and maintainers | Architecture, decisions, invariants, and implementation rationale. |
| `docs/development/` | Contributors and maintainers | Contribution, release, testing, and maintenance workflows. |

Keep these boundaries intact. User tasks and tutorials do not belong in design
documents. Design rationale does not belong in the README except for brief
context that links to the detailed design. Human-facing documents must not
embed agent-only instructions.

## Ownership and maintenance

The author of a code, workflow, or interface change owns the matching
documentation update. Reviewers check that the documentation is accurate,
appropriately located, linked from its index, and written in English.

Update documentation in the same change when it affects:

- command-line behavior, configuration, output, errors, or diagnostics;
- GitHub Action behavior, inputs, outputs, or setup;
- public installation or basic usage information;
- architecture, invariants, dependencies, or a significant implementation
  decision; or
- contributor, test, build, release, or maintenance workflows.

If no documentation changes are needed, state that assessment in the change
description when it would not be obvious to a reviewer.

## Terminology

- **User documentation** explains how to use `ecci`.
- **Design documentation** explains why the system is structured or behaves as
  it does.
- **Development documentation** explains how people contribute and maintain the
  repository.
- **Agent guidance** is operational direction for coding agents and is kept
  separate from human documentation.

Use the project name `ecci`, the spelling `.editorconfig`, and the terms
"GitHub Action" and "command-line interface (CLI)" consistently. Define an
unfamiliar acronym on first use in a page.

## Markdown, linking, and review

Use ATX headings, sentence-style heading capitalization, fenced code blocks
with a language where known, and relative links. Keep pages focused, use
descriptive link text, and update the relevant directory index when adding a
page. Prefer links over duplicating content; the README links to primary docs,
and each documentation area has an index.

During review, verify that commands, paths, option names, and links are correct;
that the page serves its declared audience; and that it contains neither stale
behavior nor content belonging to another documentation area.
