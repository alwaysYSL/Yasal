**Language:** English · [Indonesia](README.id.md)

# Yasal (يسأل)

A Windows command palette written in Rust. Search apps, files, and commands from the keyboard.

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078d7.svg)](https://microsoft.com/windows)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

## Features

- **Global hotkey** (`Alt + Space`): open the palette from anywhere. The hotkey is configurable in `settings`.
- **App launcher**: Start Menu/Desktop shortcuts and registered AppsFolder apps.
  - `Enter` opens. `Ctrl + Enter` runs a shortcut as Administrator when available. `Tab` opens the Action Panel.
- **Calculator**: type `= 25 * 4 + 10` or `100 / 4`. `Enter` copies the result.
  - Real-number division (`10/4` = `2.5`), floating-point noise rounded away (`0.1+0.2` = `0.3`). Use `.` as the decimal separator. Functions such as `= math::sqrt(16)` work.
  - **Offline unit conversion**: `5 km to mi`, `72 f in c`, `1.5 jam ke menit`, `2 gb to mb`. Length, mass, volume, temperature, time, data, area, speed; English and Indonesian unit names. `Enter` copies the number, `Ctrl + Enter` copies it with the unit. Data uses SI prefixes (1 KB = 1000 B); use `KiB`, `MiB`, `GiB` for base 1024.
- **Web search and URL handler**: prefix `??` to search Google, DuckDuckGo, or Bing. Typing a URL opens it.
  - **Aliases**: add `web_aliases` entries to `config.json`, e.g. `{"keyword": "yt", "url": "https://www.youtube.com/results?search_query={query}"}`, then type `yt rust tutorial`. Keywords: letters, digits, `-`, `_`, max 16 characters. URLs must start with `http://` or `https://` and contain `{query}`. Reserved keywords: `file`, `cb`, `ip`, `network`, `w`, `settings`. Restart Yasal after editing `config.json`.
- **Shell commands**: prefix `>` to run a PowerShell command.
  - `Enter` opens a new PowerShell window. `Ctrl + Enter` runs it in the background (15 s limit) and shows up to 200 output lines in the palette. `Enter` on an output line copies that line; `Enter` on the summary row copies all output.
- **Windows system commands**: lock, Sleep, empty Recycle Bin, Shutdown, Restart, Sign Out. Shutdown, Restart, and Sign Out ask for confirmation. Emptying the Recycle Bin does not.
- **Settings view**: type `settings` to filter and change settings. Keyboard and mouse both work.
- **Windows theme**: follows the Windows app mode and DWM accent color. Dark/Light can also be set in Settings.
- **Automatic memory trimming**: when the palette is hidden, Yasal asks Windows to shrink the working set via `EmptyWorkingSet`. Actual numbers depend on the system; working set is not Private Bytes.

### More search and utilities

- **File search**: `file <query>`. For global queries of 3+ characters that are not URLs, Yasal also tries file search when there is no local match other than the Web fallback.
  - Yasal looks for `es.exe` (Everything CLI) in `PATH` and common install locations. Without `es.exe`, or if Everything fails, the first search builds a recursive index of Documents, Desktop, and Downloads, capped at 15,000 entries and 4 MiB of paths.
  - Search folders are set in `settings` under **Folder Pencarian File**: `Enter` opens a folder picker; `Enter` on a folder row removes it (max 8). With an empty list, Everything searches its whole index, or the fallback index uses Documents, Desktop, and Downloads. With folders set, Everything uses `es.exe -path` and the fallback index reads only those folders.
  - Search runs in the background with a 150 ms debounce.
- **Window switcher**: `w` lists open windows, `w <text>` filters by title or process. `Enter` switches and restores minimized windows. `Tab` offers a close action.
- **Windows Settings search**: `$ <query>`, e.g. `$ bluetooth`.
- **Clipboard history**: `cb` or `cb <query>`. Up to 50 text items, in memory only while the plugin is enabled. Cleared when the plugin or app closes.
- **Network info**: `ip` or `network` shows local IPv4, gateway, MAC address, and public IP. The public IP is fetched from `https://api.ipify.org` in the background and cached for 60 s. This is the only outbound network request Yasal makes on its own.
- **Palette position**: monitor under the mouse cursor, falling back to the primary monitor.

## Keyboard shortcuts

| Key | Action |
|---|---|
| `Alt + Space` | Show / hide the palette |
| `↑` / `↓` | Navigate results or actions |
| `Enter` | Primary action. On calculator, network, and clipboard results, copies the value |
| `Ctrl + Enter` | Secondary action when available: Run as Admin, show file in Explorer, copy result, run `>` command and show output |
| `Tab` | Open the Action Panel when the selected item has extra actions |
| `Esc` | In Action Panel or Settings: back to search. In search: clear text, or close the palette if empty |

## Build, install, run

### Requirements

- Windows 10 version 1809 or Windows 11.
- Rust toolchain, the `x86_64-pc-windows-gnu` target, and a compatible MinGW-w64 linker. The build target is set in `.cargo/config.toml`.

### Build and test

From the repository root:

```powershell
rustup target add x86_64-pc-windows-gnu
cargo run
cargo test
cargo build --release
```

The release binary is at `target/x86_64-pc-windows-gnu/release/yasal.exe`.

### Manual install

There is no installer or release package yet. After a release build:

```powershell
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\Yasal'
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item 'target\x86_64-pc-windows-gnu\release\yasal.exe' (Join-Path $installDir 'yasal.exe')
Start-Process -FilePath (Join-Path $installDir 'yasal.exe')
```

Yasal runs in the background with a system tray icon; press `Alt + Space` to open the palette. To start with Windows, enable `Mulai bersama Windows` in `settings` once the binary is in its final location. Windows stores the executable path in the registry, so disable startup before moving the binary.

### Configuration and local data

| Location | Contents |
|---|---|
| `%APPDATA%\Yasal\config.json` | App and plugin preferences |
| `%APPDATA%\Yasal\frecency.json` | Usage history for result ranking |
| `%LOCALAPPDATA%\Yasal\IconCache\` | App/file icon cache |
| `%LOCALAPPDATA%\Yasal\yasal_debug.log` | Diagnostic log, capped near 512 KiB before restart |

Clipboard history is never written to disk. If the local data directory is unavailable, the icon cache and log use the working directory. If the config directory is unavailable, `config.json` uses `Yasal\config.json` in the working directory and frecency is not saved.

### Uninstall

1. If `Mulai bersama Windows` is on, open Yasal and turn it off in `settings`. If Yasal is already stopped, run the executable again first.
2. Choose **Exit** from the tray icon menu.
3. Delete the folder containing `yasal.exe`.
4. Optional: delete `%APPDATA%\Yasal` and `%LOCALAPPDATA%\Yasal` to remove preferences, frecency, icon cache, and logs.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Architecture notes are in [docs/DESIGN.md](docs/DESIGN.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Yasal by you, as defined in the Apache-2.0 license, is dual licensed as above, without any additional terms or conditions.

`vendor/eframe` is a locally patched copy of `eframe` 0.29.1 (MIT OR Apache-2.0, © Emil Ernerfeldt). Its original license files are included in that directory.
