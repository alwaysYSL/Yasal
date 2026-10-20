# 🌸 Yasal — Rencana Audit & Perbaikan UI/UX serta Fungsionalitas (Phase 1 MVP)

## Hasil Audit Masalah & Akar Masalah

Berdasarkan pengujian dan screenshot dari pengguna (`media_1789745055759.png` dan `media_1789745287288.png`):

### 1. Posisi Window Tidak di Tengah (Random / Top-Left)
* **Penyebab:** Jendela eframe belum mengatur koordinat eksplisit monitor utama saat window handle pertama kali dibuat.
* **Solusi Sesuai Rencana (DESIGN.md Sec 9):** Posisikan jendela selalu di monitor utama (primary display), centered horizontal (`(screen_w - 650) / 2`), dan sedikit di atas tengah vertikal (`(screen_h - 420) / 3`) menggunakan Win32 `SetWindowPos`.

### 2. Layout Structure & Status Bar Docking
* **Penyebab:** Seluruh elemen (Search Bar, Result List, Status Bar) dirender berurutan di dalam satu `CentralPanel` tanpa docking. Akibatnya:
  * Jika hanya 2 item: Status bar melayang canggung di tengah jendela tepat di bawah item ke-2.
  * Jika 8 item: Status bar terdorong ke bawah keluar dari area jendela (terpotong).
* **Solusi Sesuai Rencana:** Menggunakan struktur panel baku `egui`:
  * `TopBottomPanel::top`: Search Bar (fixed di atas).
  * `TopBottomPanel::bottom`: Status Bar (ter-dock kokoh di paling bawah jendela).
  * `CentralPanel`: Area scrollable untuk Result List / Action Panel / Settings View di antara keduanya.

### 3. Missing Glyphs (Kotak `□`) pada Status Bar
* **Penyebab:** Font bawaan sistem tidak memiliki glyph untuk unicode `⏎` dan `⇥`.
* **Solusi:** Ganti dengan teks tombol yang universal dan elegan: `Enter`, `Ctrl+Enter`, `Tab`, `Esc` dengan pemisah bullet `·`.

### 4. Keyboard Focus Hilang Setelah Keluar dari Action Panel (`Tab` / `Esc`)
* **Penyebab:** Saat menekan `Esc` dari Action Panel, `view_mode` kembali ke `Search` namun `self.search_state.request_focus = true` tidak diset.
* **Solusi:** Set `self.search_state.request_focus = true` di semua handler transisi mode (`Esc`, `Tab`, dan open window).

### 5. View Settings Tidak Terbuka
* **Penyebab:**
  1. Saat mengetik `settings`, string filter yang dioper ke `render_settings_view` bernilai `"settings"`, sehingga menyaring 0 item (karena judul item adalah "Global Hotkey", "Tema Tampilan", dsb).
  2. Saat klik menu tray "Settings", window show me-reset `view_mode` kembali ke `Search`.
* **Solusi:**
  1. Jika query adalah `"settings"`, bersihkan filter menjadi `""` sehingga seluruh daftar pengaturan langsung muncul.
  2. Jika query berupa `"settings hotkey"`, saring pengaturan dengan kata `"hotkey"`.
  3. Perbaiki routing event menu tray `TrayAction::OpenSettings`.

### 6. Ekstraksi Native Icon Aplikasi Windows
* **Solusi:** Implementasikan ekstraksi icon asli Windows via Win32 `SHGetFileInfoW` (`SHGFI_ICON | SHGFI_LARGEICON`) dan disk cache di `%LOCALAPPDATA%\Yasal\IconCache\` sesuai [DESIGN.md](file:///f:/Projek/Yasal/docs/DESIGN.md) section 2.

---

## Proposed Changes

### Component 1: Platform & Win32 Helpers (`src/platform/win32.rs`)
#### [MODIFY] [win32.rs](file:///f:/Projek/Yasal/src/platform/win32.rs)
- Tambahkan `center_window_on_primary_display(hwnd: isize, width: i32, height: i32)`.
- Tambahkan helper ekstraksi icon native aplikasi `extract_and_cache_icon(app_path: &Path, out_png_path: &Path) -> bool`.

### Component 2: UI Panels & Layout (`src/app.rs`, `src/ui/status_bar.rs`, `src/ui/settings_view.rs`)
#### [MODIFY] [app.rs](file:///f:/Projek/Yasal/src/app.rs)
- Ubah arsitektur rendering ke `TopBottomPanel::top` (Search Bar), `TopBottomPanel::bottom` (Status Bar), dan `CentralPanel` (Results / Content).
- Perbaiki logika query filter untuk mode Settings.
- Pastikan `request_focus = true` di seluruh transisi keyboard `Esc` dan `Tab`.
- Integrasikan pemanggilan `center_window_on_primary_display`.

#### [MODIFY] [status_bar.rs](file:///f:/Projek/Yasal/src/ui/status_bar.rs)
- Bersihkan karakter unicode yang hilang (`⏎`, `⇥`) menjadi tombol teks bersih (`Enter`, `Tab`, `Ctrl+Enter`, `Esc`).

#### [MODIFY] [settings_view.rs](file:///f:/Projek/Yasal/src/ui/settings_view.rs)
- Perbaiki filter query pencarian settings agar selalu menampilkan seluruh item ketika pengguna berada di mode settings default.

---

## Verification Plan

### Automated Tests
```powershell
cargo test
cargo build --release
```

### Manual Verification
1. Jalankan `yasal.exe`.
2. Tekan `Alt + Space`:
   - Jendela muncul **tepat di tengah horizontal monitor utama**, sedikit di atas tengah vertikal.
   - Search bar di atas, status bar menempel di paling bawah jendela tanpa tertutup atau melayang.
   - Tidak ada karakter kotak `□` pada status bar.
3. Tekan `Tab` pada item aplikasi:
   - Action Panel muncul.
   - Tekan `Esc` -> Kembali ke Search Result, dan **pengguna bisa langsung mengetik tanpa harus klik mouse**.
4. Ketik `settings`:
   - Daftar pengaturan (Global Hotkey, Startup, Tema, Plugin, Search Engine, Info) langsung muncul lengkap dan bisa di-toggle menggunakan `Enter`/`Space`.
5. Klik kanan tray icon -> "Settings":
   - Palette langsung terbuka di mode Settings.
