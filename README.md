# displayz

[![MIT License](https://img.shields.io/crates/l/displayz)](https://choosealicense.com/licenses/mit/) [![Continuous Integration](https://github.com/michidk/displayz/actions/workflows/ci.yaml/badge.svg)](https://github.com/michidk/displayz/actions/workflows/ci.yaml) [![rust docs](https://docs.rs/displayz/badge.svg)](https://docs.rs/displayz/latest/displayz/) [![Crates.io](https://img.shields.io/crates/v/displayz)](https://crates.io/crates/displayz)
[![Chocolatey](https://img.shields.io/chocolatey/v/displayz?include_prereleases)](https://community.chocolatey.org/packages/displayz)

A CLI tool and library to control display settings on Windows written in Rust.

## Installation

### Chocolatey

Install [displayz using Chocolatey](https://community.chocolatey.org/packages/displayz) on Windows:

```sh
choco install displayz
```

### Cargo

Install [displayz using Cargo](https://crates.io/crates/displayz) on Windows:

```sh
cargo install displayz
```

## Usage

### Commandline

After installation, the `displayz` command will be available.

Use the following command to access the help:

```sh
displayz --help
```

The following subcommands are available:

- `info`: Displays information about all connected displays.
- `set-primary --id <id>`: Sets the display with the specified ID as the primary display.
- `primary <properties>`: Sets the primary display properties.
- `properties --id <id> <properties>`: Sets the display properties for the specified ID.

The `<properties>` argument can be multiple (but at least one and max one per kind) of:

- `--position <x>,<y>`: Sets the position of the display.
- `--resolution <width>x<height>`: Sets the resolution of the display.
- `--orientation <orientation>`: Sets the orientation of the display.
  - Orientation can be either `Default`, `UpsideDown`, `Right` or `Left`.
- `--fixedoutput <fixed output mode>`: Sets the fixed output mode of the display.
  - The mode can be one of `Default`, `Stretch` or `Center`.
- `--frequency <frequency>`: Sets the refresh rate of the display.

### Rust Library

See the examples in the [examples/](examples/) folder and the [documentation](https://docs.rs/displayz/latest/displayz/) on how to use the library.

## Tests

CI compiles the display integration tests but does not run them. They require
real Windows displays and can change the primary display or its orientation.
Run them on a Windows desktop, one at a time:

```sh
cargo test -- --ignored --test-threads=1
```

## Releases

The workflows use the shared [Orcastrate templates](https://github.com/michidk/workflows).
Configure `RELEASE_TOKEN` and crates.io Trusted Publishing using the
[setup guide](https://github.com/michidk/workflows#rust-releases).
The Trusted Publisher uses owner `michidk`, repository `displayz`, workflow
`publish.yml`, and environment `publish-crates.io`.

Run **Bump Version & Tag** on `main` with the next `X.Y.Z` version. After
verification and the Windows build, **Release** creates a draft with git-cliff
release notes. Publishing the draft triggers **Publish to crates.io**.
