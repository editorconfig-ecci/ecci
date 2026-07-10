# ecci

`ecci` is a Rust-based checker for whether files conform to their applicable
`.editorconfig` settings. It is also packaged as a Docker-based GitHub Action.

## Installation

Install a Rust toolchain, clone this repository, and build the workspace:

```sh
cargo build --workspace
```

## Basic usage

Run the current command-line prototype from the repository root:

```sh
cargo run --package ecci
```

The prototype currently reads the `.editorconfig` configuration for
`Cargo.toml` and prints its resolved indentation style.

## Documentation

- [User documentation](docs/user/README.md)
- [Technical design documentation](docs/design/README.md)
- [Development documentation](docs/development/README.md)
- [Documentation governance](docs/design/documentation-governance.md)
