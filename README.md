# meiga

[![CI](https://github.com/NewHouses/meiga/actions/workflows/ci.yml/badge.svg)](https://github.com/NewHouses/meiga/actions/workflows/ci.yml)

A lightweight terminal for Windows, written in Rust, with performance as the top priority.

> **Status:** early development. `meiga.exe` opens a window with a demo: the same text at three scales, drawn with DirectWrite and a glyph cache, and rectangles that you draw by dragging the mouse, so that you can see that each frame presents only the areas that changed. It uses no CPU while idle or minimized; there is no terminal yet.

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
- [PowerShell 7](https://learn.microsoft.com/powershell/scripting/install/install-powershell-on-windows) (`pwsh`) for the scripts in `tools\`. Windows PowerShell 5.1 does not run them: it only prints the `pwsh` command that does.

```powershell
cargo build --release
```

To build the zip file that is distributed (`dist\meiga-<version>-x86_64-pc-windows-msvc.zip`), install [cargo-about](https://github.com/EmbarkStudios/cargo-about) with `cargo install --locked cargo-about --features cli` and run:

```powershell
.\tools\package.ps1
```

Every change must pass these checks ([cargo-deny](https://github.com/EmbarkStudios/cargo-deny) is installed with `cargo install --locked cargo-deny`). CI runs them on every pull request. Run them from PowerShell 7:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
.\tools\check-docs.ps1
.\tools\check-powershell.ps1
.\tools\check-imports.ps1
.\tools\check-window.ps1
.\tools\check-present.ps1
.\tools\perf\test-measure.ps1
.\tools\perf\test-scenarios.ps1
```

`check-imports.ps1`, `check-window.ps1` and `check-present.ps1` check the release build: build it first with `.\tools\package.ps1` or `cargo build --release`. `check-window.ps1` and `check-present.ps1` start `meiga.exe`, so its window appears for a moment; do not click or type in it while `check-present.ps1` draws in it. `test-scenarios.ps1` also checks the bash version of the load scenarios in WSL, and skips it with a warning when WSL has no bash 5. On Linux, CI checks the bash scripts with [ShellCheck](https://www.shellcheck.net/) and runs them.

The latest build of each pull request can be downloaded from the artifacts of its CI run.

## Measuring

`tools\perf\measure.ps1` measures the CPU, wakeups, memory and threads of a terminal and of the parts of Windows that put it on the screen (the Desktop Window Manager and Remote Desktop). It works with any terminal and does not need administrator rights. The results are printed and saved as CSV in `.perf\`, which is not committed:

```powershell
.\tools\perf\measure.ps1 WindowsTerminal -Duration 60 -Label idle
```

`tools\perf\run-scenario.ps1` measures a terminal while it runs a load scenario: a spinner (`spinner`), an AI tool that streams its answers (`ai-stream`), or printing a 50 MiB file (`bigfile`). Each scenario has a PowerShell and a bash version, for WSL tabs. Run `run-scenario.ps1` from another terminal: it shows the command to run in the measured one, measures 3 windows of 60 s, and prints a summary with a verdict on how stable the results are:

```powershell
.\tools\perf\run-scenario.ps1 spinner WindowsTerminal
.\tools\perf\run-scenario.ps1 ai-stream warp -Shell wsl
```

`tools\perf\measure-startup.ps1` measures how long `meiga.exe` takes to open its window: it starts it 20 times, closes it each time, and prints the median and the 95th percentile of the time until the window is painted:

```powershell
.\tools\perf\measure-startup.ps1
```

`tools\perf\measure-frames.ps1` measures what the frames of meiga cost to meiga, to the Desktop Window Manager and to Remote Desktop: it starts `meiga.exe` maximized with `--bench partial`, which moves a small square without a pause, then with `--bench full`, which changes the whole window at each frame, and then with `--bench text`, which fills the whole window with text that changes at each frame. Do not use the computer while it measures:

```powershell
.\tools\perf\measure-frames.ps1
```

Run `Get-Help` with `-Detailed` on each script for all the options, for example `Get-Help .\tools\perf\run-scenario.ps1 -Detailed`.

## Documentation

Detailed documentation of the architecture, the technologies and the code, written in Galician, is in [docs/](docs/README.md). The architecture decision records are in [docs/adr/](docs/adr/README.md).

## Contributing

This is a personal project and external contributions are not accepted at the moment.

## Security

See [SECURITY.md](SECURITY.md).

## License

Copyright (c) 2026 Serxio Simons Casasnovas. All rights reserved. See [LICENSE](LICENSE).
