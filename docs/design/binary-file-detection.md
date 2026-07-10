# Binary-file detection

## Decision

`ecci` will initially use an embedded, deterministic binary-file classifier. It
will be mandatory as part of file selection, will read only a bounded prefix,
and will explicitly recognise UTF-16 before applying a NUL-byte heuristic.
Files classified as binary are skipped without producing EditorConfig
diagnostics.

The Rust Magika command-line interface (CLI) is **not a required runtime
dependency** and will not be added to the initial Docker action image. A
future Magika integration may be provided as an opt-in enhancement after a
corpus comparison demonstrates material value. It must never be the only way
to preserve UTF-16 files from binary exclusion.

This is a design decision for the planned path traversal and does not claim
that the current prototype CLI already performs binary-file detection.

## Context and requirements

EditorConfig checks such as whitespace, line ending, and final newline are not
meaningful for arbitrary binary data. Reading every selected file as text can
also create noisy diagnostics or unusable output. Conversely, a detector that
treats every NUL byte as binary would incorrectly skip ordinary UTF-16 text,
because UTF-16 text commonly contains NUL bytes.

The existing checker already validates and decodes configured `utf-16le` and
`utf-16be` files, accepts a matching BOM when present, and accepts a missing
BOM when the configured byte order is known. Selection must preserve that
capability. Detection is a selection decision; it is not a substitute for the
configured `charset` validation.

The initial detector has these requirements:

- It must make no network request and add no external executable at runtime.
- It must operate on a bounded sample, initially the first 8 KiB, so that
  discovery cost is bounded regardless of file size.
- A UTF-16 BOM, or a sample whose resolved `.editorconfig` charset is
  UTF-16, must classify as text before NUL-byte analysis. The latter keeps a
  malformed configured UTF-16 file selected for charset diagnostics.
- An unreadable file remains an I/O error, not a binary file. Empty files are
  text. A detector failure must not silently make a file disappear.
- The result must be deterministic across the supported platforms and must
  not depend on file extensions.

## Candidates and evaluation

| Candidate | Benefits | Costs and risks | Result |
| --- | --- | --- | --- |
| Embedded rules: BOM/charset protection, then sampled control-byte and NUL-byte test | No additional image or process; predictable, fast, portable; policy stays in the `ecci` codebase | Cannot identify every binary format and needs a deliberately maintained threshold | Select for the initial implementation |
| Rust Magika CLI bundled in the Docker image and spawned by `ecci` | Broad learned content-type recognition; reports a stable `is_text` field and can inspect batches | Adds a second executable, model/runtime supply chain, process startup, architecture-specific delivery, and failure policy | Do not bundle initially; reconsider as optional enhancement |
| Link Magika as a Rust library | Avoids a child process and permits session reuse | The library requires ONNX Runtime provisioning; expands the `ecci` dependency and build surface | Do not adopt initially |
| Platform `file` command | Familiar and potentially already installed | Magic databases and output differ by base image/platform; it is absent from the current runtime image and its prose/MIME output is not a stable API | Reject |

### Magika assessment

Magika is a stable Rust CLI and library using the `standard_v3_3` model.
Its project describes a model loaded once per process, about 5 ms inference per
file thereafter, and near-constant work based on a limited file sample. These
figures are useful potential benefits for a long-lived or batched integration,
but do not include `ecci` process startup, model load, JSON parsing, or a
separate child-process launch. They are therefore not an end-to-end latency
budget for this action.

The release tagged `cli/v1.1.0` illustrates the distribution cost: its
compressed CLI archives are about 7.7 MiB (Apple arm64), 8.6 MiB (Linux x86_64
GNU), 9.1 MiB (Linux arm64 GNU), and 10.1 MiB (Windows x86_64). The embedded
ONNX model source is 1 MiB, but a complete CLI also brings the executable and
its ONNX Runtime arrangement. The current `ecci` runtime is Alpine 3.19;
Magika's published Linux artifacts are GNU-targeted, not Alpine/musl-targeted.
Copying one release artifact into that image is consequently not a portable
Docker solution. Building it in the existing builder would instead lengthen
the build and introduce the ONNX Runtime build/provisioning path.

Docker actions execute the image's Linux userspace. To support both common
GitHub-hosted Linux runners and ARM runners, a bundled Magika solution needs
multi-architecture image builds and per-platform verification. Direct `ecci`
CLI distribution would additionally need native artifacts for macOS and
Windows. The Magika release set demonstrates that these artifacts are distinct;
they must be pinned and checksum-verified rather than selected by an
unversioned installer at Docker build time.

Magika is Apache-2.0 licensed, compatible with this repository's intended use,
but introducing it still requires preserving its license notice and recording
the pinned CLI, model, and ONNX Runtime versions in release materials. Magika
maintains separate model and CLI/binding changelogs, so updates must be treated
as a combined behaviour change, not as a routine package refresh.

For a future optional integration, `ecci` should invoke one pinned Magika
process for a batch of paths, request structured JSON Lines output, and consume
the documented `is_text` boolean rather than descriptions or MIME types.
Magika itself advises labels rather than descriptive or MIME output for
automation. The adapter must retain the embedded UTF-16 protection as a
pre-check and map only an explicit `is_text: false` result to exclusion.

Sources: [Magika repository and license](https://github.com/google/magika),
[CLI and binding status](https://securityresearch.google/magika/cli-and-bindings/overview/),
[latest release assets](https://github.com/google/magika/releases/tag/cli/v1.1.0),
and [Magika guidance for automated output](https://securityresearch.google/magika/additional-resources/faq/).

## Initial embedded classifier

For each regular file, the traversal layer resolves its `.editorconfig`
settings, reads at most 8 KiB, and classifies the sample in this order:

1. An empty sample is text.
2. `FF FE` and `FE FF` (UTF-16 little- and big-endian BOMs) are text.
3. If the resolved `charset` is `utf-16le` or `utf-16be`, classify as text.
   A valid sample can subsequently be decoded with that byte order, including
   one with no BOM. A malformed sample also remains selected so that the normal
   charset check, rather than binary exclusion, reports the encoding error.
4. Otherwise, a sample containing NUL or a disallowed C0 control byte is
   binary. Permit tab, line feed, carriage return, and form feed, which occur
   in ordinary text.
5. A sample with no such byte is text, whether or not it is valid UTF-8.
   This avoids treating a legacy single-byte text encoding as binary; charset
   validation remains responsible for reporting an incompatible configured
   encoding.

The precise control-byte set and sample limit are implementation constants with
unit tests. The classifier returns `Text` or `Binary`; read errors are returned
separately. No `Unknown => binary` fallback is permitted. If a later
optional Magika adapter fails to execute, times out, returns malformed output,
or omits a requested path, `ecci` must emit one clear tool diagnostic and use
the embedded classifier for that path. It must not skip the path solely because
Magika failed. A future strict mode, if wanted, must be an explicit CLI option
and is listed as an open question below.

## Docker, GitHub Action, and CI impact

The initial decision requires no Dockerfile, `action.yml`, dependency, or
GitHub Action change. It keeps the present Alpine runtime image small relative
to a second packaged CLI and makes the action self-contained once the `ecci`
image is built. CI needs only the Rust unit/integration tests and the existing
Docker build check; it neither downloads a model nor depends on an upstream
classification service.

If Magika becomes optional later, CI must build every published action
architecture, assert the pinned artifact checksum and executable version, and
run a fixture corpus through both the embedded and Magika paths. Docker build
inputs must be pinned by version and digest/checksum; `curl | sh`, `cargo
install` without a lock/pin, and runtime downloads are not acceptable. Model
and CLI updates require a reviewed compatibility test run and an image-size
measurement recorded in the update pull request.

## Acceptance criteria and test plan

The initial implementation is acceptable only when all of the following are
covered by automated tests:

- UTF-16LE and UTF-16BE text with a BOM classify as text and reach the normal
  checker.
- Valid UTF-16LE and UTF-16BE text without a BOM classify as text when the
  applicable `.editorconfig` explicitly sets the corresponding charset.
- A UTF-16 file is not excluded merely because its sample contains NUL bytes.
- NUL-containing binary fixtures, including a short file and a file whose
  binary marker appears within the 8 KiB sample, classify as binary and yield
  no content-rule diagnostics.
- UTF-8 text, ASCII text with tabs/newlines, an empty file, and a non-UTF-8
  single-byte text fixture are not excluded by binary detection.
- A malformed or odd-length UTF-16 fixture does not panic. With a UTF-16
  configuration it remains selected so the existing charset diagnostic can be
  reported; without that configuration it follows the ordinary sampled rule.
- Permission/read errors and optional-detector errors are reported and do not
  silently skip a path.
- Directory traversal tests prove binary selection is independent of filename
  extension, and a Docker-action smoke test exercises the same fixtures.

Before reconsidering Magika, add a versioned evaluation corpus containing the
fixtures above plus representative images, archives, executables, generated
files, and text in supported encodings. Measure action image compressed and
uncompressed size, cold action elapsed time, and a batched repository scan on
both `linux/amd64` and `linux/arm64`. The proposal must demonstrate fewer
incorrect exclusions without a UTF-16 regression.

## User-controlled overrides

`ecci` will add an `.ecciignore` file as a lightweight remedy for files that
should not be checked, including binary formats the embedded detector does not
recognise. It uses `.gitignore`-compatible patterns, is discovered in the same
directory hierarchy as `.gitignore`, and has higher precedence than
`.gitignore`. The `.ecciignore` file itself is never checked.

A negated `.ecciignore` pattern (for example, `!fixtures/example.txt`) is a
**force-check** rule: when it is the final matching `.ecciignore` rule, `ecci`
checks the regular, readable file even when the embedded binary classifier
returns `Binary`. This is the escape hatch for text files incorrectly excluded
by the classifier. It does not make directories or special files checkable,
and it does not suppress normal I/O or charset errors.

Use the Rust [`ignore` crate](https://docs.rs/ignore/) for traversal and
`.gitignore`-compatible parsing. `WalkBuilder` can discover
`.ecciignore` through `add_custom_ignore_filename`; its custom ignore files
take precedence over standard ignore files. The traversal implementation must
also retain the `.ecciignore` match result with the candidate path. A walker
alone only tells `ecci` that the path is eligible for traversal; it does not
preserve whether eligibility came from a negated pattern. Use the crate's
lower-level `gitignore::Gitignore` matcher (or an equivalent wrapper) to record
the final ignore/whitelist match and set `force_check` for a final whitelist.

The implementation task must include user documentation under `docs/user/`
covering location, pattern syntax, precedence with `.gitignore`, negation and
force-check semantics, and its interaction with binary detection. The README
should remain limited to a link if a user-facing configuration page is added.

## Open questions

- Should a future strict optional-Magika mode fail the command when Magika is
  unavailable, or is fallback-only behaviour sufficient for all users?
- Should malformed UTF-16 with a configured UTF-16 charset always bypass
  binary exclusion, as proposed, to guarantee a charset diagnostic?
- What false-positive/false-negative rate on the evaluation corpus would
  justify the additional image size and supply-chain maintenance?
- If Magika is adopted, should it be a separately tagged action image or an
  action input selecting an image that already contains the pinned tool?
