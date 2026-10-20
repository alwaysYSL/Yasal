# 🌸 Yasal — UI/UX Design

> Dokumen ini melengkapi `archive/implementation_plan.md`. Fokusnya murni pada keputusan desain antarmuka dan interaksi, hasil dari sesi diskusi UI/UX.

---

## 1. Prinsip Desain

1. **Comfortable without excess** — palette harus cepat dipindai oleh power-user, tetapi tetap memberi ruang visual yang cukup agar hasil dan kontrol terasa jelas.
2. **Predictable over expressive** — tidak ada perubahan ukuran/reflow yang mengejutkan saat navigasi keyboard. Semua baris hasil punya tinggi yang sama.
3. **Native, bukan web-app** — mengikuti konvensi Windows (accent color sistem, dialog native untuk konfirmasi destruktif) supaya terasa seperti bagian dari OS, bukan aplikasi Electron yang menempel.
4. **Zero-cost visual** — setiap elemen visual (icon asli, preview) harus lazy-loaded dan tidak mengorbankan target RAM (< 25 MB idle) atau startup time (< 200 ms).

---

## 2. Layout Hasil Pencarian

**Pola: Flat uniform list** (bukan top-match emphasis, bukan list+preview panel).

| Properti | Nilai |
|---|---|
| Row height | **44px**, fixed di semua baris |
| Vertical row gap | **4px** |
| Icon size | **24×24px**, rounded square container (radius 8px) |
| Selection corner radius | **14px** |
| Max visible items | 8 (virtualized list, hanya render yang visible) |
| Selection highlight | Background tint aksen + garis aksen tipis di kiri (bukan resize font/icon) |
| Info per baris | Icon kiri → title (14px) → subtitle kontekstual (path file / kategori, 12px, muncul jika perlu disambiguasi) → badge kategori kanan (12px, muted) |

**Alasan menolak alternatif:**
- *Top-match emphasis* (Spotlight-style) memakan ruang vertikal berharga dari budget 8-slot, dan bikin tinggi baris tidak konsisten → menyulitkan virtualisasi list.
- *List + preview panel* (Alfred-style) butuh window lebih lebar dari 650px, bertentangan dengan filosofi ultra-ringan.

### Icon

- **Icon asli aplikasi**, bukan generik, via `SHGetImageList` dengan `SHIL_JUMBO` (256×256, di-downscale ke 24px untuk ketajaman di layar HiDPI/scaling 150-200%).
- **Lazy extraction**: saat indexing awal hanya simpan path. Icon diekstrak hanya untuk item yang tampil di hasil (8-10 item visible), dijalankan di thread terpisah (non-blocking).
- **Disk cache**: `%LOCALAPPDATA%\Yasal\IconCache\`, key = hash(`path + last_modified_time`) agar otomatis invalidate saat aplikasi ter-update.
- **Placeholder state**: kotak rounded abu-abu netral tampil dulu, icon di-swap masuk begitu ekstraksi/cache-read selesai.
- **File individual** (hasil dari Everything): icon tipe file dari shell (`SHGetFileInfo`), di-cache in-memory (jumlah jenis terbatas, sering dipakai ulang).
- **Item non-file** (Web Search, Calculator, System Commands, Network Info): icon simbolik/generik dalam container seragam yang sama dengan icon asli, supaya tetap "duduk" rapi berdampingan.

---

## 3. Search Bar & Mode/Prefix System

- Indikator mode aktif (`=`, `>`, `??`, `cb`, dst): **icon di sisi kiri search bar berubah** sesuai mode yang terdeteksi dari prefix yang diketik. Contoh: masuk mode kalkulator (`=`) → icon search berganti jadi icon kalkulator.
- Placeholder text juga ikut berubah kontekstual per mode ("Ketik ekspresi matematika..." saat mode kalkulator).
- Search field memakai permukaan yang sedikit terangkat dan bentuk kapsul; tidak ada garis pemisah antara header pencarian dan daftar hasil.
- Tinggi search field **50px**, radius **25px** (kapsul), tanpa outline terpisah, dengan margin horizontal konten **16px**.
- **Lifecycle & Reset Input:**
  - Setiap kali window dibuka kembali via `Alt+Space`, search bar **selalu dalam keadaan kosong/bersih (fresh input)**.
  - Jika search bar berisi teks, menekan `Esc` pertama kali akan membersihkan teks (kembali ke state awal/frecency). Menekan `Esc` saat search bar sudah kosong akan menutup window.

---

## 4. State-State UI

| State | Perilaku |
|---|---|
| **Empty** (window baru dibuka, belum ada ketikan) | Tampilkan **4-5 recent/frequently used apps**, memanfaatkan algoritma frecency yang sudah direncanakan untuk ranking — tanpa biaya komputasi tambahan. |
| **No results** | Pesan sederhana **"Tidak ada hasil"**, tanpa saran fallback ke web search otomatis. |
| **Loading** (mis. calculator ekspresi kompleks, network info fetch) | Indikator kecil di baris terkait, bukan blocking seluruh UI. |
| **Konfirmasi aksi destruktif** (Shutdown/Restart/Sign Out) | **Native Windows dialog** terpisah, bukan modal overlay di dalam window Yasal. |
| **Settings** | **View built-in di dalam palette** — diakses dengan mengetik `settings` atau via tray icon. Disajikan sebagai daftar baris interaktif (44px rows, jarak 4px) yang dapat dicari/difilter dan diatur murni via keyboard. |

---

## 5. Action Panel (`Tab`) & Interaksi Keyboard

### Mekanisme Action Panel (In-place Replacement / Raycast-style)
Saat user menyorot sebuah item di hasil pencarian dan menekan tombol `Tab`:
1. **Transisi In-place**: Result list utama seketika digantikan oleh daftar aksi kontekstual untuk item tersebut (menggunakan struktur baris 44px dengan jarak 4px yang seragam).
2. **Breadcrumb Search Bar**: Placeholder/badge di search bar berubah menjadi format breadcrumb: `[Nama Item] > Ketik aksi...` (misal `Visual Studio Code > Ketik aksi...`).
3. **Daftar Aksi Contoh (Aplikasi)**:
   - `🛡️ Jalankan sebagai Administrator` (`Ctrl+Enter`)
   - `📁 Buka Lokasi File di Explorer`
   - `📋 Salin Path File`
   - `📌 Pin ke Taskbar / Start Menu`
4. **Navigasi**:
   - `↑` / `↓`: Memilih aksi
   - `Enter`: Eksekusi aksi terpilih
   - `Esc` atau `Backspace` (saat query aksi kosong): Kembali seketika ke daftar hasil pencarian utama.

### Tabel Interaksi Keyboard

| Key | Kondisi / Aksi |
|---|---|
| `↑` / `↓` | Navigasi antar baris item / aksi / setting |
| `Enter` | Eksekusi aksi utama item terpilih (atau ubah/toggle nilai di view Settings) |
| `Ctrl+Enter` | **Aksi sekunder default** per kategori:<br>• **App**: Run as Administrator<br>• **File**: Buka folder lokasi file di Explorer<br>• **Calculator / Network Info**: Salin hasil ke clipboard |
| `Tab` | Buka **Action Panel** (in-place replacement) untuk item terpilih |
| `Esc` | • **Jika di Action Panel**: Kembali ke hasil pencarian<br>• **Jika ada teks di Search Bar**: Bersihkan teks input<br>• **Jika Search Bar kosong**: Tutup window |

---

## 6. Status Bar (Contextual Dynamic Hints)

Status bar berada di baris paling bawah window sebagai panduan pintasan keyboard yang **berubah secara kontekstual** mengikuti item yang sedang disorot dan state view yang aktif:

| State / Item yang Disorot | Petunjuk Status Bar |
|---|---|
| **Aplikasi (App)** | `↑↓ Navigasi · ⏎ Buka · Ctrl+⏎ Run as Admin · ⇥ Aksi · Esc Tutup` |
| **File / Dokumen** | `↑↓ Navigasi · ⏎ Buka · Ctrl+⏎ Buka Folder · ⇥ Aksi · Esc Tutup` |
| **Kalkulator** | `↑↓ Navigasi · ⏎ Salin Hasil · Esc Tutup` |
| **Web Search** | `↑↓ Navigasi · ⏎ Buka di Browser · Esc Tutup` |
| **Di dalam Action Panel (`Tab`)** | `↑↓ Pilih Aksi · ⏎ Jalankan · Esc Kembali` |
| **Di dalam View Settings** | `↑↓ Pilih · ⏎ Ubah / Toggle · Esc Kembali` |

---

## 7. View Settings Built-in

Settings disajikan sebagai **interactive list items** (re-use layout baris 44px dengan jarak 4px), sehingga tidak membutuhkan mouse atau form rendering terpisah:

- **Searchable**: Search bar menyaring daftar pengaturan secara langsung (contoh: ketik `hotkey` langsung memfilter baris hotkey).
- **Struktur Baris Pengaturan**:
  - `⌨️ Global Hotkey` — Nilai: `Alt + Space` *(Tekan Enter untuk merekam hotkey baru)*
  - `🚀 Jalankan saat Windows Startup` — Nilai: `[ON / OFF]` *(Tekan Enter/Spasi untuk toggle)*
  - `🎨 Tema Tampilan` — Nilai: `[Auto (Sistem) | Gelap | Terang]` *(Tekan Enter untuk rotasi pilihan)*
  - `🔌 Plugin: Everything File Search` — Nilai: `[ON / OFF]`
  - `🔌 Plugin: Kalkulator` — Nilai: `[ON / OFF]`
  - `🔌 Plugin: Clipboard History` — Nilai: `[ON / OFF]`
  - `🌐 Search Engine Default` — Nilai: `[Google | DuckDuckGo | Bing]`
  - `ℹ️ Tentang Yasal` — Nilai: `v0.1.0 · Ringan & Cepat`

---

## 8. Theming

- **Integrasi Tema Windows**:
  - Secara default mengikuti mode Windows (Light/Dark) secara otomatis via API DWM (`DWMWA_USE_IMMERSIVE_DARK_MODE`).
  - Mendukung opsi manual override di Settings: `Auto (Sistem)`, `Gelap`, atau `Terang`.
- **Accent Color**: Mengambil accent color Windows aktif secara real-time dari sistem pengguna.
- **Transparansi / Material**: Windows 11 Acrylic/Mica blur (fallback ke semi-transparent dark/light yang solid & elegan bila Acrylic tidak didukung oleh hardware/driver).

---

## 9. Window Behavior

| Properti | Nilai |
|---|---|
| Ukuran | Kartu lebar **650px**, tinggi dinamis hingga **500px** untuk menampilkan maksimal 8 baris. Jendela OS 18px lebih besar di setiap sisi (margin transparan untuk bayangan) |
| Margin konten | 16px horizontal, 8px vertikal |
| Posisi | **Monitor tempat kursor mouse berada** (bukan selalu monitor utama), center horizontal, sedikit di atas tengah vertikal. Tepi atas dihitung dari tinggi maksimum kartu sehingga tidak bergeser saat jumlah hasil berubah. Skala DPI diambil dari monitor tujuan |
| Corner radius | 26px (frame), 25px (search field), 14px (highlight), 8px (wadah ikon) |
| Border frame | Stroke tipis 1px mengikuti accent color Windows |
| Shadow | Bayangan lembut dilukis egui di margin transparan sekeliling kartu (bayangan DWM native tidak mengikuti sudut 26px). DWM rounding dan border sistem dimatikan |
| Animasi | Fade in 100ms (easing keluar). Fade out 50ms belum diimplementasikan; jendela langsung disembunyikan |

---

## 10. Ringkasan Keputusan (Quick Reference)

- [x] Layout hasil: flat uniform list, 44px row dengan jarak 4px, 24px icon asli (lazy extraction + disk/memory cache)
- [x] Mode indicator: icon kiri search bar berubah per prefix
- [x] Search bar lifecycle: fresh input on open, 2-step Esc (clear text → close window)
- [x] Empty state: recent/frequently used (frecency-based)
- [x] No-results: pesan polos, tanpa saran web otomatis
- [x] Settings: built-in searchable list view (44px rows, 4px gap, keyboard-driven)
- [x] Action Panel: in-place list replacement on `Tab`, breadcrumb di search bar
- [x] Status Bar: contextual dynamic hints sesuai item & state
- [x] Theme: sinkronisasi tema Windows + manual override (Auto/Dark/Light) + Accent Color DWM
- [x] Konfirmasi destruktif: native Windows dialog
- [x] Multi-monitor: selalu primary display
- [x] Ctrl+Enter: aksi sekunder default per kategori

---

## 11. Phase Selanjutnya (Roadmap)

- **Phase 2**:
  - File Search (Everything IPC integration)
  - Color Picker UI (magnifier loupe overlay & color code copy)
  - Clipboard History UI & Ring Buffer
- **Phase 3**:
  - Installer (WiX / MSIX) & Auto-updater
