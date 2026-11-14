# meiga

[![CI](https://github.com/NewHouses/meiga/actions/workflows/ci.yml/badge.svg)](https://github.com/NewHouses/meiga/actions/workflows/ci.yml)

A lightweight terminal for Windows, written in Rust, with performance as the top priority.

> **Status:** early development. `meiga.exe` is already a terminal: it runs a program in a pseudoconsole (ConPTY), by default PowerShell 7, shows its output with colours, bold and italic text, underlines, wide characters, a cursor and the title that the program sets, and sends it the keys that you type as the Windows console does, also the dead keys and AltGr of layouts such as the Spanish one. The mouse selects text, which goes to the clipboard, and pastes it, and programs that ask for the mouse, such as vim or htop, get it. The grid follows the size and the scale of the window, with no garbage left on the screen, the scrollback keeps 10 000 lines, with a thin scroll bar, and the text can be zoomed. Each frame presents only the cells that changed, and it uses no CPU while idle or minimized. This first milestone, a terminal for a single shell, has passed its performance gate. Tabs come next: their logic is in place, and a console example tries it.

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

The zip file includes the ConPTY of Windows Terminal (`conpty.dll` and `OpenConsole.exe`), which `package.ps1` takes from `tools\fetch-conpty.ps1`.

Every change must pass these checks ([cargo-deny](https://github.com/EmbarkStudios/cargo-deny) is installed with `cargo install --locked cargo-deny`). CI runs them on every pull request. Run them from PowerShell 7:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
.\tools\fetch-conpty.ps1
cargo test --workspace
cargo deny check
.\tools\check-docs.ps1
.\tools\check-powershell.ps1
.\tools\check-imports.ps1
.\tools\check-window.ps1
.\tools\check-present.ps1
.\tools\check-keyboard.ps1
.\tools\check-mouse.ps1
.\tools\check-resize.ps1
.\tools\perf\test-measure.ps1
.\tools\perf\test-scenarios.ps1
```

`fetch-conpty.ps1` downloads the ConPTY that meiga ships, the NuGet package `Microsoft.Windows.Console.ConPTY` at a pinned version, checks its SHA-256 and its Microsoft signatures, and extracts it to `target\conpty\`. It downloads it only once. The tests of `meiga-win` run programs in it, and fail with a message that asks for the script when it is missing. `.\tools\fetch-conpty.ps1 -CheckLatest` tells whether nuget.org has a newer stable version; CI checks it every Monday.

`check-imports.ps1`, `check-window.ps1`, `check-present.ps1`, `check-keyboard.ps1`, `check-mouse.ps1` and `check-resize.ps1` check the release build: build it first with `.\tools\package.ps1`, which also puts the ConPTY of meiga next to it. `check-window.ps1` runs cmd in meiga, and checks that cmd sets the title of the window and ends with it; `check-present.ps1` runs a PowerShell script in meiga that writes to it step by step, and checks the frames; `check-keyboard.ps1` types keys in the window of meiga, with the Spanish or the US keyboard layout, and checks what a PowerShell script in its terminal reads; `check-mouse.ps1` clicks, drags and pastes in the window of meiga, and checks the clipboard and what PowerShell scripts in its terminal read; `check-resize.ps1` resizes the window of meiga, also while the message box that asks before pasting is open, and checks that its screen has the rows of the console buffer of ConPTY, and then checks the scrollback, the scroll bar and the zoom. Their windows appear for a moment: do not click or type while `check-present.ps1`, `check-keyboard.ps1`, `check-mouse.ps1` or `check-resize.ps1` runs. With `-Layout`, `check-keyboard.ps1` may change the keyboard layout of the whole session until it ends. `check-mouse.ps1` and `check-resize.ps1` use the clipboard, and put its text back at the end; `check-resize.ps1` also moves the mouse pointer, and puts it back. `test-scenarios.ps1` also checks the bash version of the load scenarios in WSL, and skips it with a warning when WSL has no bash 5. On Linux, CI checks the bash scripts with [ShellCheck](https://www.shellcheck.net/) and runs them.

To see what a program sends through the ConPTY, run it in the console example of `meiga-win`. It prints each block of output with the control characters in sight, and sends each line that you type, followed by Enter; Ctrl+Z and Enter close the session:

```powershell
cargo run -p meiga-win --example conpty                     # cmd, in the ConPTY of meiga
cargo run -p meiga-win --example conpty -- --system         # cmd, in the ConPTY of Windows
cargo run -p meiga-win --example conpty -- pwsh -NoLogo     # any command line
```

With `--record <file>`, it also records the output, block by block, and with `--quiet` it only prints the end of the session. The console example of `meiga-core` gives a recording to the terminal of meiga as fast as it can, and prints how fast the terminal read it and, with `--screen`, what it shows at the end:

```powershell
cargo run --release -p meiga-win --example conpty -- --record ola.mrec cmd /c echo ola
cargo run --release -p meiga-core --example replay -- ola.mrec --screen
```

Another console example of `meiga-core` tries the logic of the tabs, which `meiga.exe` does not use yet. Type commands such as `new`, `close b`, `move a 3`, `activate a` or `next`, and it prints the tab bar as text, with the active tab between brackets; `help` lists the commands:

```powershell
cargo run -p meiga-core --example tabs
```

The latest build of each pull request can be downloaded from the artifacts of its CI run.

## Running

```powershell
.\target\release\meiga.exe                     # PowerShell 7 or, if it is not installed, Windows PowerShell
.\target\release\meiga.exe cmd /k title Hello  # any program, with its arguments
.\target\release\meiga.exe pwsh -NoProfile -File tools\terminal-sample.ps1
```

The command line is `meiga [--trace-frames <file>] [--marks] [--] [<program> [<arguments>]]`: the options of meiga go before the program, everything after the program is for it, and `--` ends the options. `--marks` draws a magenta border around the areas that each frame presents. Type in the window as in any terminal: the keys reach the program as in the Windows console, and Alt+F4 and Alt+Space close the window and open its menu, as in any other window. When the program ends, meiga shows its exit code and keeps the window open until you close it.

Drag with the left button to select text, double-click to select a word or a whole path, and triple-click to select a line: the selection goes to the clipboard when the button goes up. Ctrl+C or Ctrl+Shift+C copy the selection and remove it (without a selection, Ctrl+C reaches the program), Esc removes it, and Ctrl+V, Ctrl+Shift+V or a right click paste the clipboard. Before pasting more than one line into a program that did not turn on bracketed paste, such as PowerShell, meiga asks. Programs that ask for the mouse, such as vim or htop, get the clicks and the wheel: hold Shift to select text in them.

The wheel scrolls back through the last 10 000 lines, as do Shift+Page Up and Shift+Page Down (with or without Ctrl), Ctrl+Shift+Up and Ctrl+Shift+Down (a line), and Ctrl+Shift+Home and Ctrl+Shift+End (the top and the bottom). The scroll bar on the right widens under the pointer: drag its thumb, or click above or below it to move a page. Output does not move the view while you read; typing a key for the program, or pasting, brings it back to the bottom. Dragging a selection above or below the window scrolls on its own. On the alternate screen, which full-screen programs use, those keys go to the program. Ctrl with + or - (= and - on a US keyboard, and also those of the numeric keypad), or Ctrl and the wheel, make the text bigger or smaller, from 6 to 72 points, and Ctrl+0 brings it back to 12 points.

meiga uses the `conpty.dll` and `OpenConsole.exe` next to `meiga.exe`, as in the zip file, and the ConPTY of Windows when they are missing. `package.ps1` also copies them next to `target\release\meiga.exe`. `tools\terminal-sample.ps1` writes a sample of colours, styles and wide characters, to check a terminal by eye.

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

With `-Start`, it starts a terminal that runs the program of its command line, such as meiga, with the scenario as its program, and closes it at the end. With `-Maximized` too, it starts it maximized:

```powershell
.\tools\perf\run-scenario.ps1 bigfile -Start .\target\release\meiga.exe
.\tools\perf\run-scenario.ps1 spinner -Start .\target\release\meiga.exe -Maximized
```

`tools\perf\measure-startup.ps1` measures how long `meiga.exe` takes to open its window, with its shell started: it starts it 20 times, closes it each time, and prints the median and the 95th percentile of the time until the window is painted:

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
