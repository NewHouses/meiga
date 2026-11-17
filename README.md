# meiga

[![CI](https://github.com/NewHouses/meiga/actions/workflows/ci.yml/badge.svg)](https://github.com/NewHouses/meiga/actions/workflows/ci.yml)

A lightweight terminal for Windows, written in Rust, with performance as the top priority.

> **Status:** early development. `meiga.exe` is already a terminal: it runs a program in a pseudoconsole (ConPTY), by default PowerShell 7, shows its output with colours, bold and italic text, underlines, wide characters, a cursor and the title that the program sets, and sends it the keys that you type as the Windows console does, also the dead keys and AltGr of layouts such as the Spanish one. The mouse selects text, which goes to the clipboard, and pastes it, and programs that ask for the mouse, such as vim or htop, get it. The grid follows the size and the scale of the window, with no garbage left on the screen, the scrollback keeps 10 000 lines, with a thin scroll bar, and the text can be zoomed. Each frame presents only the cells that changed, and it uses no CPU while idle or minimized. This first milestone, a terminal for a single shell, has passed its performance gate. Tabs are on their way: each tab runs its own shell, and the tabs that are not shown only spend what reading the output of their programs costs. A tab bar at the top shows them with the titles of their programs, and the mouse or keyboard shortcuts open, show and close them. The menu of a tab, with the right button or from the keyboard, duplicates it and closes it, the other tabs or the tabs on its right. The button + opens a tab with the default profile, and the ▾ next to it opens the menu of the profiles: PowerShell 7, Windows PowerShell, Command Prompt, the WSL distributions and Git Bash, which meiga finds without starting any process, and the ones of an optional configuration file. Keyboard shortcuts also show a tab by its place, open a profile or duplicate a tab, and the configuration file can change or free every key that meiga takes. When the program of a tab ends with exit code 0, the tab closes, and the last one closes the window; any other end leaves the tab, which tells how the program ended, and then Enter starts it again, and Ctrl+D closes the tab.

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
.\tools\check-tabs.ps1
.\tools\check-tabbar.ps1
.\tools\check-menu.ps1
.\tools\check-profiles.ps1
.\tools\check-shortcuts.ps1
.\tools\check-exit.ps1
.\tools\perf\test-measure.ps1
.\tools\perf\test-scenarios.ps1
```

`fetch-conpty.ps1` downloads the ConPTY that meiga ships, the NuGet package `Microsoft.Windows.Console.ConPTY` at a pinned version, checks its SHA-256 and its Microsoft signatures, and extracts it to `target\conpty\`. It downloads it only once. The tests of `meiga-win` run programs in it, and fail with a message that asks for the script when it is missing. `.\tools\fetch-conpty.ps1 -CheckLatest` tells whether nuget.org has a newer stable version; CI checks it every Monday.

`check-imports.ps1`, `check-window.ps1`, `check-present.ps1`, `check-keyboard.ps1`, `check-mouse.ps1`, `check-resize.ps1`, `check-tabs.ps1`, `check-tabbar.ps1`, `check-menu.ps1`, `check-profiles.ps1`, `check-shortcuts.ps1` and `check-exit.ps1` check the release build: build it first with `.\tools\package.ps1`, which also puts the ConPTY of meiga next to it. They start meiga with an empty folder as its `APPDATA`, so that it has its default values, whatever your `config.toml` has; `check-profiles.ps1`, `check-shortcuts.ps1` and `check-exit.ps1` give it a `config.toml` of their own. `check-window.ps1` runs cmd in meiga, and checks that cmd sets the title of the window and ends with it; `check-present.ps1` runs a PowerShell script in meiga that writes to it step by step, and checks the frames; `check-keyboard.ps1` types keys in the window of meiga, with the Spanish or the US keyboard layout, and checks what a PowerShell script in its terminal reads; `check-mouse.ps1` clicks, drags and pastes in the window of meiga, and checks the clipboard and what PowerShell scripts in its terminal read; `check-resize.ps1` resizes the window of meiga, also while the message box that asks before pasting is open, and checks that its screen has the rows of the console buffer of ConPTY, and then checks the scrollback, the scroll bar and the zoom; `check-tabs.ps1` opens, shows and closes tabs in the window of meiga with their shortcuts, and checks what a hidden tab does and does not do; `check-tabbar.ps1` clicks the tab bar of meiga, and checks what each click does and draws, that a program that changes its title nonstop, or gives the same one again and again, does not wake the window up more than 1000 times per second, and that the bar keeps the mouse while one of its buttons is down, also when Ctrl+Tab shows another tab; `check-menu.ps1` opens the menu of a tab of meiga with the right button and from the keyboard, and checks where it opens, what each change draws, what its items do, the keys that it keeps from the program, and when it closes; `check-profiles.ps1` gives meiga a `config.toml` with errors, and checks the message box that tells them, the environment, the command line and the folder of the profiles that meiga finds, and the menu of the profiles, with the keyboard and with the mouse; `check-shortcuts.ps1` gives meiga a `config.toml` with shortcuts, some of them wrong, and checks the message box that tells the errors, that Ctrl+T, Ctrl+W, AltGr and the keys that the file frees reach the program, and what the shortcuts of the tabs and the profiles do, also while their keys repeat, and that a key that went down for the program keeps reaching it while it repeats with Ctrl down; `check-exit.ps1` gives meiga a `config.toml` whose profiles run a PowerShell script, ends that script with exit codes, and checks which tabs close, also while a message box is open, and that only the menu of a tab closes with it, the line that tells how a program ended, what Enter, Ctrl+D, Restart tab and the other keys do then, and a program that cannot start. Their windows appear for a moment: do not click or type while `check-present.ps1`, `check-keyboard.ps1`, `check-mouse.ps1`, `check-resize.ps1`, `check-tabs.ps1`, `check-tabbar.ps1`, `check-menu.ps1`, `check-profiles.ps1`, `check-shortcuts.ps1` or `check-exit.ps1` runs. With `-Layout`, `check-keyboard.ps1` may change the keyboard layout of the whole session until it ends. `check-mouse.ps1`, `check-resize.ps1`, `check-tabs.ps1` and `check-exit.ps1` use the clipboard, and put its text back at the end; `check-resize.ps1`, `check-tabbar.ps1`, `check-menu.ps1`, `check-profiles.ps1`, `check-shortcuts.ps1` and `check-exit.ps1` also move the mouse pointer, and put it back. `test-scenarios.ps1` also checks the bash version of the load scenarios in WSL, and skips it with a warning when WSL has no bash 5. On Linux, CI checks the bash scripts with [ShellCheck](https://www.shellcheck.net/) and runs them.

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

Another console example of `meiga-core` tries the logic of the tabs by hand. Type commands such as `new`, `close b`, `move a 3`, `activate a` or `next`, and it prints the tab bar as text, with the active tab between brackets; `help` lists the commands:

```powershell
cargo run -p meiga-core --example tabs
```

The latest build of each pull request can be downloaded from the artifacts of its CI run.

## Running

```powershell
.\target\release\meiga.exe                     # the default profile: PowerShell 7 or, if it is not installed, Windows PowerShell
.\target\release\meiga.exe cmd /k title Hello  # any program, with its arguments
.\target\release\meiga.exe pwsh -NoProfile -NoExit -File tools\terminal-sample.ps1
```

The command line is `meiga [--trace-frames <file>] [--marks] [--] [<program> [<arguments>]]`: the options of meiga go before the program, everything after the program is for it, and `--` ends the options. `--marks` draws a magenta border around the areas that each frame presents. Type in the window as in any terminal: the keys reach the program as in the Windows console, and Alt+F4 and Alt+Space close the window and open its menu, as in any other window. When the program of a tab ends with exit code 0, as `exit` in PowerShell, its tab closes, and closing the last tab closes the window, so `meiga cmd /c dir` opens and closes. Any other end, or a program that cannot start, leaves the tab, which tells how it ended: press Enter to start the program again in the same tab, with the same profile, command line and folder, or Ctrl+D to close the tab. What the last program wrote goes to the scrollback.

The tab bar at the top of the window shows a tab for each shell, with the title that its program sets or, without one, the name of its profile. Click a tab to show it; click its × or middle-click it to close it; and click + to open a tab with the default profile. When the tabs do not fit, they all shrink alike. Right-click a tab to open its menu, or press the Menu key or Shift+F10 to open the menu of the tab that is shown: Duplicate tab opens a tab with the same profile, program and folder right after it; Restart tab, once the program of the tab has ended, starts it again in the tab; and Close tab, Close other tabs and Close tabs to the right close tabs. In the menu, the arrows, Home and End move the highlight, Enter chooses, and Esc or a click outside it closes it. Ctrl+Shift+T opens a tab with the default profile, at the end, and shows it, and Ctrl+Shift+1 to Ctrl+Shift+9 open the profile at that place of the menu of the profiles; Ctrl+Tab or Ctrl+Page Down and Ctrl+Shift+Tab or Ctrl+Page Up show the next and the previous tab, Ctrl+1 to Ctrl+8 the tab at that place, and Ctrl+9 the last one; Ctrl+Shift+D duplicates the tab that is shown; and Ctrl+Shift+W closes it and ends its programs. Closing the last tab closes the window. Ctrl+T and Ctrl+W still reach the program, and so does AltGr, which writes characters such as `@` or `€`, and is never a shortcut. Only the first tab, and its copies, run the program of the command line. The programs of the tabs that are not shown keep running, but meiga only draws their titles in the bar: the window takes the size of a tab when it shows it, if the window changed meanwhile. The zoom is the same for all the tabs.

The ▾ next to the button +, or Ctrl+Shift+Space, opens the menu of the profiles, with the default one in bold: choose one to open it in a tab at the end. meiga finds them without starting any process, each time that the menu opens: PowerShell 7, Windows PowerShell and Command Prompt, which start in your home folder, the WSL distributions that are installed, which start in their Linux home folder, and Git Bash. Their programs get `TERM_PROGRAM=meiga`, `TERM_PROGRAM_VERSION` and `COLORTERM=truecolor`, which `WSLENV` also passes to the programs of WSL, and not the variables of Windows Terminal (`WT_SESSION` and `WT_PROFILE_ID`). An optional file, `%APPDATA%\meiga\config.toml`, changes, hides or adds profiles, and chooses the default one, which the button +, Ctrl+Shift+T and the first tab open:

```toml
default_profile = "Ubuntu"

[[profiles]]
name = "Command Prompt"
hidden = true

[[profiles]]
name = "Build"
command = 'pwsh -NoExit -File "%USERPROFILE%\build.ps1"'
directory = 'C:\src\meiga'
```

An entry with the name of a profile that meiga finds changes its command line or its folder, or hides it; one with a new name and a `command` adds a profile after them; and one without a `command`, nor a profile of its name, is ignored, so the same file works on computers without the same shells. `command` and `directory` expand `%NAME%` variables, and a `directory` of `~` is the home folder. meiga reads the file when it starts. If it has errors, a message box in front of the window tells where each one is, and meiga ignores what is wrong, or the whole file if it is not TOML, and uses its default values instead.

Every key that meiga takes is a shortcut, also the ones that copy, paste, scroll back, zoom and open the menus, and the table `[shortcuts]` of `config.toml` adds, changes or frees them. A shortcut is written as in Windows Terminal: the modifier keys `ctrl`, `alt` and `shift` and a key, joined with `+`, where a key is a letter, a digit, `f1` to `f24`, `numpad0` to `numpad9`, or a name such as `enter`, `tab`, `space`, `pgup`, `pgdn`, `home`, `end`, `left`, `up`, `plus` or `minus`. Keys go by the key and not by the character that it writes, so `ctrl+shift+1` is the same shortcut with any keyboard layout:

```toml
[shortcuts]
"ctrl+t" = "new_tab"            # another shortcut for an action
"ctrl+6" = "none"               # Ctrl+6 reaches the program again
"alt+o" = "close_other_tabs"
```

The actions are `new_tab`, `close_tab`, `next_tab`, `previous_tab`, `tab_1` to `tab_N`, `last_tab`, `profile_1` to `profile_N`, `profile_menu`, `tab_menu`, `duplicate_tab`, `close_other_tabs`, `close_tabs_to_the_right`, `copy`, `paste`, `scroll_up`, `scroll_down`, `scroll_page_up`, `scroll_page_down`, `scroll_to_top`, `scroll_to_bottom`, `zoom_in`, `zoom_out` and `zoom_reset`, and `none` frees the keys, which then reach the program. A shortcut to a tab or a profile that is not there does nothing. A wrong entry, or a second one with the same keys, is an error in the message box, and meiga ignores it.

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
