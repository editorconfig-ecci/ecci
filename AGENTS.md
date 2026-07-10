# Coding-agent instructions

## Project

`ecci` is a Rust workspace that checks `.editorconfig` settings and provides a
GitHub Action.

## Working rules

- Write repository documentation, source-code comments, issues, pull-request
  titles and descriptions, and review comments in English.
- Communicate with users in their preferred language. Kanban task titles,
  prompts, and progress reports are user-facing board communication, so they
  may use the language the user used in the conversation; do not require
  English for them under the repository documentation policy. This exception
  does not apply to persistent repository artifacts, which must be written in
  English.
- Keep `README.md` brief and human-facing: purpose, installation, basic usage,
  and links only. Put user guidance in `docs/user/`, design rationale in
  `docs/design/`, and contributor workflows in `docs/development/`.
- Read [documentation governance](docs/design/documentation-governance.md)
  before creating or restructuring documentation.
- Before proceeding, ask the user a concise, user-facing clarifying question
  when a material ambiguity remains in the requirements, intended behavior,
  acceptance criteria, or priority and no safe assumption can be made. State
  the decision the user needs to make. Do not silently choose product behavior
  on the user's behalf.
- Resolve routine, low-risk implementation details from established repository
  conventions without unnecessary questions.
- Update affected documentation when behavior, configuration, diagnostics,
  action inputs, architecture, or contributor workflow changes.
- For Rust changes, run the relevant tests; normally use `cargo test --workspace`.
  Report commands run and results, or explain why verification was not run.
- Keep changes scoped to the request. Do not alter unrelated code, tests,
  generated bindings, dependencies, or formatting.
