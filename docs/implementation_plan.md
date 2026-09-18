# 🌸 Yasal — Lightweight Windows Command Palette

> **Yasal** (يسأل) — *"Bertanya, Mencari Tahu"* (Arab Klasik) · *"Mahkota Bunga"* (Persia Kuno)

Command palette ultra-ringan untuk Windows, dibangun dengan **Rust**. Pengganti PowerToys Command Palette yang boros RAM (~740 MB). Target: **< 25 MB RAM**.

---

## Keputusan yang Sudah Final

> [!NOTE]
> **Nama**: **Yasal** — gabungan nama pembuat dan orang spesial. Secara etimologi: *"Bertanya/Mencari Tahu"* (Arab) dan *"Mahkota Bunga"* (Persia). Sangat cocok untuk search tool.

> [!IMPORTANT]
> **Hotkey**: Default `Alt + Space` — **configurable** dari Settings. User bisa ganti ke kombinasi apapun (Ctrl+Space, Win+Alt+Space, dll). `RegisterHotKey` mengintercept di level OS sehingga tidak konflik dengan app manapun.

> [!NOTE]
> **File Search**: Menggunakan **[Everything](https://www.voidtools.com/)** by voidtools sebagai backend (gratis, ~15 MB RAM, indexing instan via NTFS journal). Jika Everything tidak terinstall → fallback ke `walkdir` scan direktori umum.

> [!NOTE]
> **Color Picker**: Ditunda ke **Phase 2**. Phase 1 fokus fitur text-based.

---

## Proposed Changes

### Arsitektur Tingkat Tinggi

```
┌─────────────────────────────────────────────────────────────┐
│                      Yasal Process                          │
│                     (Single Binary)                          │
│                                                             │
│  ┌──────────────┐    ┌──────────────────────────────────┐   │
│  │  System Tray  │    │         Event Loop                │   │
│  │  (tray-icon)  │◄──►│  global-hotkey → toggle window   │   │
│  └──────────────┘    └──────────┬───────────────────────┘   │
│                                 │                            │
│                                 ▼                            │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Popup Window (egui/eframe)               │   │
│  │  ┌──────────────────────────────────────────────┐    │   │
│  │  │  🔍 Search Bar (fuzzy input)                  │    │   │
│  │  ├──────────────────────────────────────────────┤    │   │
│  │  │  📋 Result List (dynamic, max 8 items)        │    │   │
│  │  │  ├─ 🖥️ Visual Studio Code          (App)     │    │   │
│  │  │  ├─ 📁 Documents/project.rs        (File)    │    │   │
│  │  │  ├─ 🔢 = 142.5                     (Calc)    │    │   │
│  │  │  └─ ⚡ Shutdown                    (System)  │    │   │
│  │  └──────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                   Plugin Modules                        │ │
│  │  ┌─────────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐ │ │
│  │  │App      │ │File  │ │Calc  │ │System│ │Clipboard │ │ │
│  │  │Launcher │ │Search│ │      │ │Cmds  │ │History   │ │ │
│  │  └─────────┘ └──────┘ └──────┘ └──────┘ └──────────┘ │ │
│  │  ┌─────────┐ ┌──────┐ ┌──────┐ ┌──────┐              │ │
│  │  │Web      │ │Shell │ │Win   │ │Network│              │ │
│  │  │Search   │ │Cmd   │ │Settings│ │Info  │              │ │
│  │  └─────────┘ └──────┘ └──────┘ └──────┘              │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

### Tech Stack & Dependencies

```toml
[package]
name = "yasal"
version = "0.1.0"
edition = "2021"

[dependencies]
# === GUI & Windowing ===
egui = "0.29"
eframe = { version = "0.29", default-features = false, features = ["wgpu", "default_fonts"] }
window-vibrancy = "0.5"              # Windows 11 Acrylic/Mica blur

# === Hotkey & System Tray ===
global-hotkey = "0.6"                # System-wide Alt+Space
tray-icon = "0.19"                   # System tray icon + menu

# === Windows Native APIs ===
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_Graphics_Dwm",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_Shell",
    "Win32_System_Com",
    "Win32_System_Power",
    "Win32_NetworkManagement_IpHelper",
    "Win32_Networking_WinSock",
] }

# === Search & Matching ===
nucleo-matcher = "0.3"               # Fuzzy matching (from Helix editor)
walkdir = "2.5"                      # Start Menu recursive scan
everything-ipc = "0.2"              # voidtools Everything file search

# === Calculator ===
evalexpr = "11.3"                    # Math expression parser

# === Clipboard ===
clipboard-master = "4.0"            # Clipboard change listener
arboard = "3.4"                      # Clipboard read/write

# === Utilities ===
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt", "macros", "time"] }
open = "5"                           # Open URLs/files with default app
chrono = "0.4"                       # Timestamps
dirs = "5"                           # User directory paths

# === Optimized Release ===
[profile.release]
opt-level = "z"                      # Optimize for size
lto = true                           # Link-time optimization
codegen-units = 1                    # Single codegen unit
strip = true                         # Strip debug symbols
panic = "abort"                      # Abort on panic (smaller binary)
```

**Kenapa `egui` + `eframe`?**
| Kriteria | egui | slint | tauri |
|----------|------|-------|-------|
| RAM idle | ~18-35 MB ✅ | ~12-25 MB ✅ | ~65-110 MB ❌ |
| Dev speed | Cepat (immediate mode) ✅ | Moderate | Cepat (web) |
| Styling flexibility | Custom theme ✅ | Built-in Fluent | Unlimited (CSS) |
| Acrylic/Mica | Via `window-vibrancy` ✅ | Manual Win32 | Via `window-vibrancy` |
| Binary size | ~3-5 MB ✅ | ~2-4 MB | ~8-15 MB |
| Plugin extensibility | Mudah (Rust traits) ✅ | Moderate | Sangat mudah |

`egui` dipilih karena balance terbaik antara ringan, cepat develop, dan mudah di-customize.

---

### Struktur Project

```
yasal/
├── Cargo.toml
├── build.rs                          # Embed app icon, Windows manifest
├── assets/
│   ├── icon.ico                      # System tray icon
│   └── icon.png                      # Window icon
├── src/
│   ├── main.rs                       # Entry point, event loop, hotkey
│   ├── app.rs                        # eframe::App implementation
│   ├── config.rs                     # Settings (hotkey, theme, enabled plugins)
│   ├── theme.rs                      # Custom egui style (dark, Fluent-inspired)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── search_bar.rs             # Input field + prefix detection
│   │   ├── result_list.rs            # Scrollable result items
│   │   └── status_bar.rs             # Bottom hints (shortcuts, mode)
│   ├── core/
│   │   ├── mod.rs
│   │   ├── plugin.rs                 # Plugin trait definition
│   │   ├── matcher.rs                # Fuzzy matching with nucleo
│   │   ├── indexer.rs                # App indexer (Start Menu + UWP)
│   │   └── dispatcher.rs             # Routes query → correct plugin(s)
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── app_launcher.rs           # #1  Launch installed apps
│   │   ├── file_search.rs            # #2  Search files (Everything IPC)
│   │   ├── web_search.rs             # #3+7 Web search + URI handler
│   │   ├── win_settings.rs           # #5  ms-settings: navigation
│   │   ├── shell_command.rs          # #6  Run shell commands
│   │   ├── calculator.rs             # #8  Math expressions
│   │   ├── color_picker.rs           # #15 Screen color picker (Phase 2)
│   │   ├── system_commands.rs        # #16-21 Shutdown/restart/lock/sleep/etc
│   │   ├── network_info.rs           # #24 IP/Network info
│   │   └── clipboard_history.rs      # #25 Clipboard history
│   └── platform/
│       ├── mod.rs
│       ├── hotkey.rs                  # Global hotkey registration
│       ├── tray.rs                    # System tray icon + menu
│       ├── window.rs                  # Window show/hide, Acrylic blur
│       └── win32.rs                   # Raw Win32 API helpers
└── tests/
    ├── test_calculator.rs
    ├── test_matcher.rs
    └── test_plugins.rs
```

---

### Detail Implementasi Per Modul

#### 1. Core: Plugin Trait System

Semua fitur diimplementasi sebagai plugin yang mengikuti satu trait:

```rust
// src/core/plugin.rs

pub struct QueryResult {
    pub title: String,           // "Visual Studio Code"
    pub subtitle: String,        // "C:\Program Files\VS Code\code.exe"
    pub icon: Icon,              // Enum: App, File, Calc, System, Web, etc.
    pub score: u32,              // Fuzzy match score (higher = better)
    pub action: Box<dyn FnOnce()>, // What happens on Enter
}

pub trait Plugin: Send + Sync {
    /// Unique plugin ID
    fn id(&self) -> &str;
    
    /// Display name
    fn name(&self) -> &str;
    
    /// Optional prefix trigger (e.g., ">" for shell, "=" for calc)
    fn prefix(&self) -> Option<&str> { None }
    
    /// Query the plugin with user input, return results
    fn query(&self, input: &str) -> Vec<QueryResult>;
    
    /// Whether this plugin responds to unprefixed queries
    fn is_global(&self) -> bool { false }
}
```

**Routing Logic** (`dispatcher.rs`):
1. User types `> ipconfig` → prefix `>` detected → route to `ShellCommand` plugin only
2. User types `= 25 * 4` → prefix `=` detected → route to `Calculator` plugin only
3. User types `visual studio` → no prefix → broadcast to all `is_global()` plugins → merge & sort by score

---

#### 2. Plugin: App Launcher (`app_launcher.rs`)

```
Trigger:  Global (no prefix), user types app name
Indexing: On startup, scan Start Menu .lnk files + UWP apps
Matching: nucleo fuzzy matcher
Action:   Launch app via ShellExecuteW or explorer.exe shell:AppsFolder\{aumid}
```

- Index di-build saat startup (~50-200ms), disimpan in-memory (~1-3 MB untuk ~500 apps)
- Re-index setiap 5 menit di background thread
- Mendukung Win32 apps (.lnk) DAN UWP/Store apps (via `shell:AppsFolder`)

---

#### 3. Plugin: File Search (`file_search.rs`)

```
Trigger:  Prefix "file " atau global fallback
Backend:  Everything IPC (voidtools)
Latency:  1-5ms per query
Action:   Open file/folder atau reveal in Explorer
```

- Jika Everything tidak terinstall → fallback ke `walkdir` scan `Documents`, `Desktop`, `Downloads`
- Pencarian real-time saat user mengetik (debounce 150ms)

---

#### 4. Plugin: Web Search + URI Handler (`web_search.rs`)

```
Trigger:  Prefix "?? " untuk search, auto-detect URL
Action:   open::that(url) → buka di default browser
```

- `?? cara install rust` → `https://www.google.com/search?q=cara+install+rust`
- `https://github.com` → langsung buka URL
- `mailto:john@mail.com` → buka email client
- Search engine bisa dikonfigurasi (Google, DuckDuckGo, Bing)

---

#### 5. Plugin: Windows Settings (`win_settings.rs`)

```
Trigger:  Prefix "$ " atau global match "settings"
Action:   open::that("ms-settings:display")
```

Hardcoded map ~40 settings pages:
- `$ display` → `ms-settings:display`
- `$ wifi` → `ms-settings:network-wifi`
- `$ bluetooth` → `ms-settings:bluetooth`
- `$ update` → `ms-settings:windowsupdate`
- dll.

---

#### 6. Plugin: Shell Command (`shell_command.rs`)

```
Trigger:  Prefix "> "
Action:   Spawn PowerShell/CMD process
```

- `> ipconfig` → jalankan di terminal baru
- `> Shell:startup` → buka folder Startup
- Output ditampilkan inline jika singkat, atau buka terminal window

---

#### 7. Plugin: Calculator (`calculator.rs`)

```
Trigger:  Prefix "= " atau auto-detect math expression
Action:   Copy result ke clipboard on Enter
```

- `= 25 * 4 + 10` → `110`
- `= sqrt(256)` → `16`
- `= 0xFF` → `255`
- `15% of 200` → `30` (jika pakai kalk, atau evalexpr untuk basic math)
- Result otomatis ter-copy ke clipboard saat Enter

---

#### 8. Plugin: System Commands (`system_commands.rs`)

```
Trigger:  Global match
Action:   Win32 API calls
```

| Command | Implementation |
|---------|---------------|
| Shutdown | `ExitWindowsEx(EWX_SHUTDOWN, ...)` |
| Restart | `ExitWindowsEx(EWX_REBOOT, ...)` |
| Lock | `LockWorkStation()` |
| Sleep | `SetSuspendState(false, true, false)` |
| Sign Out | `ExitWindowsEx(EWX_LOGOFF, ...)` |
| Empty Recycle Bin | `SHEmptyRecycleBin(...)` |

- Konfirmasi dialog sebelum execute (Shutdown/Restart/Sign Out)

---

#### 9. Plugin: Network Info (`network_info.rs`)

```
Trigger:  Prefix "ip" atau "network"
Action:   Copy IP to clipboard
```

- Tampilkan: Local IPv4, Public IP (via API), MAC address, gateway
- Public IP fetch: simple HTTP GET ke `https://api.ipify.org`

---

#### 10. Plugin: Clipboard History (`clipboard_history.rs`)

```
Trigger:  Prefix "cb " atau hotkey Ctrl+Shift+V (dari palette)
Storage:  In-memory ring buffer (max 50 entries)
Action:   Paste selected item
```

- Background thread monitor via `clipboard-master` (`WM_CLIPBOARDUPDATE`)
- Deduplicate consecutive identical copies
- Tampilkan preview (truncated 80 chars) + timestamp
- History persist ke JSON file (optional, configurable)

---

#### 11. Color Picker (`color_picker.rs`) — Phase 2

```
Trigger:  Prefix "color" atau dedicated hotkey
Action:   Enter pick mode → klik screen → copy HEX/RGB
```

- Win32 GDI: `GetDC(0)` + `GetPixel(hdc, x, y)`
- Tampilkan: HEX, RGB, HSL
- Magnifier loupe (5x zoom grid) di sekitar cursor
- Di-delay ke Phase 2 karena butuh overlay rendering khusus

---

### UI Design

```
┌────────────────────────────────────────────────────┐
│  ╭────────────────────────────────────────────╮     │
│  │  🔍  Type to search...                    │     │  ← Acrylic blur background
│  ╰────────────────────────────────────────────╯     │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │ 🖥️  Visual Studio Code                      │   │  ← Selected (highlighted)
│  │     Application                              │   │
│  ├─────────────────────────────────────────────┤   │
│  │ 📁  VS Code Settings                        │   │
│  │     C:\Users\...\settings.json              │   │
│  ├─────────────────────────────────────────────┤   │
│  │ 🌐  Search "visual studio" on Google        │   │
│  │     Web Search                              │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  ↑↓ Navigate   ⏎ Open   Esc Close   Tab Switch    │  ← Status bar hints
└────────────────────────────────────────────────────┘
```

**Window Properties:**
- Ukuran: 650 × 420 px (dynamic height based on results)
- Position: Center screen, slightly above center (seperti Spotlight)
- Background: Windows 11 Acrylic blur (fallback: semi-transparent dark)
- Corner radius: 12px
- Shadow: DWM drop shadow
- Animation: Fade in 100ms, fade out 50ms

---

### Phasing

#### Phase 1 — Core MVP (Minggu 1-2)
- [x] Project setup, build system, release optimization
- [ ] Window management (show/hide, Acrylic blur, always-on-top)
- [ ] Global hotkey (Alt+Space)
- [ ] System tray icon
- [ ] Plugin trait system + dispatcher
- [ ] Search bar UI + result list
- [ ] App Launcher (Start Menu + UWP indexing)
- [ ] Calculator
- [ ] System Commands (shutdown, restart, lock, sleep, sign out, empty recycle bin)
- [ ] Web Search + URI Handler
- [ ] Shell Command runner

#### Phase 2 — Extended Features (Minggu 3-4)
- [ ] File Search (Everything IPC integration)
- [ ] Windows Settings navigation
- [ ] Clipboard History
- [ ] Network Info
- [ ] Settings UI (configure hotkey, theme, enabled plugins)
- [ ] Persist config to JSON file

#### Phase 3 — Polish (Minggu 5+)
- [ ] Color Picker with magnifier overlay
- [ ] Custom themes (light/dark)
- [ ] Auto-start on Windows login
- [ ] Auto-updater
- [ ] Installer (WiX or NSIS)

---

## Verification Plan

### Automated Tests
```bash
# Unit tests per plugin
cargo test

# Build release binary
cargo build --release

# Check binary size (target: < 5 MB)
ls -la target/release/yasal.exe

# Check RAM usage
# Launch app → Task Manager → verify < 25 MB idle
```

### Manual Verification
- [ ] Alt+Space toggles window show/hide
- [ ] Typing app name shows fuzzy results
- [ ] Enter launches selected app
- [ ] Esc atau click outside hides window
- [ ] System tray icon visible, right-click menu works
- [ ] `= 25 * 4` shows calculator result
- [ ] `> ipconfig` opens terminal with output
- [ ] `?? rust programming` opens browser search
- [ ] System commands (shutdown, lock, etc.) work with confirmation
- [ ] RAM stays < 25 MB in idle state

### Performance Targets
| Metric | Target |
|--------|--------|
| RAM (idle/tray) | < 25 MB (< 5 MB after trim) |
| RAM (popup open) | < 35 MB |
| Hotkey → visible | < 50 ms |
| Keystroke → results | < 30 ms |
| Binary size | < 5 MB |
| Startup time | < 200 ms |

---

## 🔥 Critical Performance Optimizations

Temuan penting dari riset arsitektur launcher Rust existing:

### 1. Working Set Trimming on Hide (`EmptyWorkingSet`)
Teknik kunci yang dipakai launcher ringan — saat window di-hide, paksa OS melepas halaman memori:

```rust
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::GetCurrentProcess;

pub fn trim_memory() {
    unsafe { let _ = EmptyWorkingSet(GetCurrentProcess()); }
}
```

**Efek**: Idle RAM turun dari ~22 MB → **< 5 MB** setelah window dismiss. RAM naik kembali saat popup dibuka (instant, dari page cache).

### 2. App Enumeration via `FOLDERID_AppsFolder` (Lebih Baik dari .lnk scan)
Daripada scan `.lnk` files secara manual, gunakan COM `FOLDERID_AppsFolder` yang otomatis menemukan:
- ✅ Win32 desktop apps
- ✅ UWP / Store apps (Calculator, Terminal, Settings)
- ✅ Progressive Web Apps (PWAs)
- ✅ MSIX packaged apps

Satu API untuk semua jenis app — lebih lengkap dan lebih cepat.

### 3. Lazy Loading Modules
- Plugin berat (calculator engine, file search, clipboard) di-load via `LazyLock` / `once_cell` hanya saat prefix diketik
- Icon di-cache ke disk (`%LOCALAPPDATA%\Yasal\IconCache\`) dan hanya load icon untuk 8-10 item yang visible (virtualized list)

### 4. Search Cancellation via Atomic Revision
Saat user mengetik cepat, search lama otomatis di-cancel:

```rust
static SEARCH_REVISION: AtomicU64 = AtomicU64::new(0);
// Worker thread checks: if revision changed, abort immediately
```

### 5. Frecency Ranking
Results di-rank berdasarkan kombinasi fuzzy score + seberapa sering/baru app dipakai:

$$\text{Score} = \text{FuzzyScore} \times \sum e^{-\lambda \cdot \Delta t}$$

App yang sering dibuka akan muncul lebih tinggi di hasil pencarian.

---

## Referensi: Existing Rust Launchers

| Project | Platform | Catatan |
|---------|----------|---------|
| [ZeroLaunch-rs](https://github.com/ghost-him/ZeroLaunch-rs) | Windows | Lightning-fast, privacy-first |
| [winsp](https://github.com/recregt/winsp) | Windows | Minimalist, blazing fast |
| [Loungy](https://github.com/MatthiasGrandl/Loungy) | macOS | GPU-rendered, pakai `nucleo` |
| [pop-launcher](https://github.com/pop-os/launcher) | Linux | Plugin JSON-IPC architecture |
| [anyrun](https://github.com/anyrun-org/anyrun) | Linux | In-process dynamic plugin (.dll) |
