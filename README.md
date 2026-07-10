# ecci

`ecci` is the end-user command-line interface (CLI) for checking files and
directories against their applicable [EditorConfig](https://editorconfig.org/)
settings.

## Installation

Building from source requires a current stable
[Rust toolchain](https://www.rust-lang.org/tools/install) with Cargo and the
EditorConfig Core C development library (`libeditorconfig`). On Debian or
Ubuntu, install it with `sudo apt-get install libeditorconfig-dev`; on Alpine,
use `apk add editorconfig-dev`.

Then clone the repository and build the CLI:

```sh
git clone https://github.com/editorconfig-ecci/ecci.git
cd ecci
cargo build --release --package ecci
```

## Basic usage

Run the current CLI prototype from the repository root:

```sh
cargo run --package ecci
```

The prototype currently resolves the `.editorconfig` settings for `Cargo.toml`
and prints its indentation style. It does not yet accept path arguments or
report conformance results.

## Documentation

- [CLI usage, configuration, checks, and roadmap](docs/user/cli.md)
- [Development and testing](docs/development/README.md)
- [Technical design documentation](docs/design/README.md)
- [Documentation governance](docs/design/documentation-governance.md)
- [MIT License](LICENSE)
