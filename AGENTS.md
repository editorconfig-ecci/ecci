# Coding-agent instructions

## Project

`ecci` is a Rust workspace that checks `.editorconfig` settings and provides a
GitHub Action.

## Working rules

- Write all repository documentation and documentation changes in English.
- Keep `README.md` brief and human-facing: purpose, installation, basic usage,
  and links only. Put user guidance in `docs/user/`, design rationale in
  `docs/design/`, and contributor workflows in `docs/development/`.
- Read [documentation governance](docs/design/documentation-governance.md)
  before creating or restructuring documentation.
- Update affected documentation when behavior, configuration, diagnostics,
  action inputs, architecture, or contributor workflow changes.
- For Rust changes, run the relevant tests; normally use `cargo test --workspace`.
  Report commands run and results, or explain why verification was not run.
- Keep changes scoped to the request. Do not alter unrelated code, tests,
  generated bindings, dependencies, or formatting.
