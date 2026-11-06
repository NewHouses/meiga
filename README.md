# meiga

[![CI](https://github.com/NewHouses/meiga/actions/workflows/ci.yml/badge.svg)](https://github.com/NewHouses/meiga/actions/workflows/ci.yml)

A lightweight terminal for Windows, written in Rust, with performance as the top priority.

> **Status:** early development. There is nothing usable yet.

## Goals

- Tabs like a web browser: rename them and group them.
- Mix different shells in one window, one per tab: PowerShell 7, Windows PowerShell, WSL distributions and cmd.
- A built-in folder and file explorer to change directory without typing `cd`, with recursive search under the current folder.
- Restore the previous session on start.
- Stay light: no work while idle, and a CPU renderer that only redraws what changed, so it runs well over Remote Desktop and on machines without a GPU.

## Building

Requirements:

- Windows 10 version 1809 or later.
- [rustup](https://rustup.rs/). The Rust version is pinned in `rust-toolchain.toml`, and rustup installs it automatically.
- The MSVC build tools (Visual Studio or the Build Tools with the "Desktop development with C++" workload).

```powershell
cargo build --release
```

To build the zip file that is distributed (`dist\meiga-<version>-x86_64-pc-windows-msvc.zip`), install [cargo-about](https://github.com/EmbarkStudios/cargo-about) with `cargo install --locked cargo-about --features cli` and run:

```powershell
.\tools\package.ps1
```

Every change must pass these checks ([cargo-deny](https://github.com/EmbarkStudios/cargo-deny) is installed with `cargo install --locked cargo-deny`). CI runs them on every pull request:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
.\tools\check-imports.ps1
```

The latest build of each pull request can be downloaded from the artifacts of its CI run.

## Documentation

Detailed documentation of the architecture, the technologies and the code, written in Galician, is in [docs/](docs/README.md).

## Contributing

This is a personal project and external contributions are not accepted at the moment.

## Security

See [SECURITY.md](SECURITY.md).

## License

Copyright (c) 2026 Serxio Simons Casasnovas. All rights reserved. See [LICENSE](LICENSE).
