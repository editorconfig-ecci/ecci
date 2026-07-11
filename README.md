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

Pass files or directories to the CLI. Directories are searched recursively; if
no path is supplied, the current directory is used.

```sh
cargo run --package ecci -- src tests
```

The command prints the files selected for checking. Its conformance-reporting
interface is still under development.

## Documentation

- [CLI usage, configuration, and checks](docs/user/cli.md)
- [`.ecciignore` configuration](docs/user/ecciignore.md)
- [Development and testing](docs/development/README.md)
- [Technical design documentation](docs/design/README.md)
- [Documentation governance](docs/design/documentation-governance.md)
- [MIT License](LICENSE)
