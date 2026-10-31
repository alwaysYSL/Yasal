# 🌸 Yasal — UI/UX Design

> Dokumen ini mencatat perilaku UI/UX yang tersedia saat ini. Status dan batas implementasi diverifikasi terhadap source.

---

## 1. Prinsip Desain

1. **Comfortable without excess** — palette harus cepat dipindai oleh power-user, tetapi tetap memberi ruang visual yang cukup agar hasil dan kontrol terasa jelas.
2. **Predictable over expressive** — tidak ada perubahan ukuran/reflow yang mengejutkan saat navigasi keyboard. Semua baris hasil punya tinggi yang sama.
3. **Native, bukan web-app** — mengikuti konvensi Windows (accent color sistem, dialog native untuk konfirmasi destruktif) supaya terasa seperti bagian dari OS, bukan aplikasi Electron yang menempel.
4. **Biaya visual terukur** — ikon diekstrak di background dan menggunakan cache disk. Target memori/startup belum dipublikasikan sebagai jaminan; ukur build release pada perangkat representatif sebelum menetapkan angka.

---

## 2. Layout Hasil Pencarian

**Pola: Flat uniform list** (bukan top-match emphasis, bukan list+preview panel).

| Properti | Nilai implementasi |
|---|---|
| Row height | **44px**, fixed di semua baris |
| Vertical row gap | **4px** |
| Icon area | 28×28px; ikon shell PNG 64×64 ditampilkan pada area ini |
| Selection corner radius | **14px** |
| Hasil terlihat | Maksimal 8 baris sebelum scroll. Dispatcher membatasi pencarian global hingga 8; pencarian ber-prefix tertentu bisa menghasilkan lebih banyak (misalnya pengalih jendela hingga 20). |
| Selection highlight | Background tint aksen pada seluruh baris; tanpa garis aksen di kiri |
| Info per baris | Ikon kiri → title (14px) → subtitle bila tersedia (11px) → badge kategori ber-outline (11px) |

**Alasan menolak alternatif:** top-match emphasis memberi bobot visual berbeda pada salah satu dari delapan slot; preview panel memerlukan window lebih lebar dari 650px. Desain saat ini menjaga baris seragam sesuai batas layout.

### Icon

- **Ikon shell asli**: diekstrak lewat `SHGetFileInfo` sebagai PNG 64×64 dan ditampilkan pada area 28×28px; bukan melalui `SHGetImageList`/`SHIL_JUMBO`.
- **Ekstraksi lazy**: path disimpan pada hasil aplikasi/file; ikon diekstrak di thread terpisah saat hasil dirender.
- **Cache**: PNG berada di `%LOCALAPPDATA%\Yasal\IconCache\`; key meng-hash path dan waktu modifikasi bila tersedia. Maksimal 64 tekstur ditahan di cache memori.
- **Placeholder**: tile netral tampil selama ikon dimuat atau jika ekstraksi gagal.
- **Item non-file** (web, kalkulator, perintah sistem, jaringan, dan lainnya): ikon simbolik.

---

## 3. Search Bar & Mode/Prefix System

- Indikator mode aktif (`=`, `>`, `??`, `cb`, dst): **icon di sisi kiri search bar berubah** sesuai mode yang terdeteksi dari prefix yang diketik. Contoh: masuk mode kalkulator (`=`) → icon search berganti jadi icon kalkulator.
- Placeholder text juga ikut berubah kontekstual per mode ("Ketik ekspresi matematika..." saat mode kalkulator).
- Search field memakai permukaan yang sedikit terangkat dan bentuk kapsul; tidak ada garis pemisah antara header pencarian dan daftar hasil.
- Tinggi search field **50px**, radius **25px** (kapsul), tanpa outline terpisah, dengan margin horizontal konten **16px**.
  - Setiap kali palette dibuka kembali melalui `Alt+Space`, query pencarian dibersihkan sebelum input baru.
  - Di mode pencarian, `Esc` membersihkan query bila berisi teks, lalu menutup palette bila query sudah kosong. Di Settings, `Esc` kembali ke pencarian.
---

## 4. State-State UI

| State | Perilaku |
|---|---|
| **Empty** (palette baru dibuka) | Hingga 5 item yang paling sering digunakan; sebelum ada riwayat, tampilkan hingga 5 aplikasi pertama pada indeks aplikasi. |
| **No results** | Pesan **“Tidak ada hasil”** bila tidak ada hasil yang bisa ditampilkan. Pencarian web tetap menjadi fallback berperingkat rendah untuk query global yang tidak menghasilkan item non-Web. |
| **Loading** | Baris status saat pencarian file atau informasi jaringan masih diproses di background. Kalkulator dievaluasi langsung. |
| **Konfirmasi aksi destruktif** | **Native Windows dialog** untuk shutdown/restart/sign out dan aksi yang memerlukannya. |
| **Settings** | View built-in, dibuka dengan `settings` atau dari system tray; judul/ID pengaturan dapat difilter dan baris dioperasikan dengan keyboard atau klik. |

---

## 5. Action Panel (`Tab`) & Interaksi Keyboard

### Mekanisme Action Panel (In-place Replacement / Raycast-style)
Saat user menyorot sebuah item di hasil pencarian dan menekan tombol `Tab`:
1. **Transisi In-place**: Result list utama seketika digantikan oleh daftar aksi kontekstual untuk item tersebut (menggunakan struktur baris 44px dengan jarak 4px yang seragam).
2. **Breadcrumb Search Bar**: Placeholder/badge di search bar berubah menjadi format breadcrumb: `[Nama Item] > Ketik aksi...` (misal `Visual Studio Code > Ketik aksi...`).
3. **Aksi kontekstual** bergantung pada item:
   - Shortcut aplikasi: jalankan sebagai Administrator, buka folder lokasi, atau salin path.
   - Aplikasi AppsFolder: buka daftar aplikasi Windows atau salin ID aplikasi.
   - File: tampilkan di Explorer. Jendela: tutup jendela.
4. **Navigasi**:
   - `↑` / `↓`: memilih aksi.
   - `Enter`: menjalankan aksi terpilih.
   - `Esc`: kembali ke hasil utama.
   - `Backspace`: kembali hanya bila query aksi kosong.

### Tabel Interaksi Keyboard

| Key | Kondisi / Aksi |
|---|---|
| `↑` / `↓` | Navigasi baris hasil, aksi, atau pengaturan |
| `Enter` | Eksekusi aksi utama; pada Settings mengubah pengaturan |
| `Ctrl+Enter` | Aksi sekunder bila tersedia: menjalankan shortcut aplikasi sebagai Administrator, menampilkan file di Explorer, atau menyalin hasil. AppsFolder tanpa aksi sekunder memakai aksi utama. |
| `Tab` | Buka Action Panel hanya bila item terpilih memiliki aksi tambahan |
| `Esc` | Action Panel: kembali · Settings: kembali ke pencarian · Pencarian: bersihkan teks atau tutup jika kosong |

---

## 6. Status Bar (Contextual Dynamic Hints)

Status bar berada di baris paling bawah window sebagai panduan pintasan keyboard yang **berubah secara kontekstual** mengikuti item yang sedang disorot dan state view yang aktif:

| State / Item yang Disorot | Petunjuk Status Bar |
|---|---|
| **Aplikasi** | `↑↓ Navigasi · ⏎ Buka · ⇥ Aksi · Esc Tutup` |
| **File / Folder** | `↑↓ Navigasi · ⏎ Buka · Ctrl+⏎ Explorer · Esc Tutup` |
| **Kalkulator / Clipboard / Jaringan** | `↑↓ Navigasi · ⏎ Salin · Esc Tutup` |
| **Web / Pengaturan Windows** | `↑↓ Navigasi · ⏎ Buka · Esc Tutup` |
| **Pengalih jendela** | `↑↓ Navigasi · ⏎ Beralih · ⇥ Aksi · Esc Tutup` |
| **Perintah Shell / Sistem** | `↑↓ Navigasi · ⏎ Jalankan · Esc Tutup` |
| **Action Panel** | `↑↓ Pilih aksi · ⏎ Jalankan · Esc Kembali` |
| **Settings** | `↑↓ Pilih · ⏎ Ubah · Esc Kembali` |

---

## 7. View Settings Built-in

Settings dibuka dengan `settings` atau menu system tray. Search bar menyaring judul dan ID pengaturan; baris setinggi 44px dengan jarak 4px dapat dipilih memakai panah, diaktifkan memakai `Enter`, atau diklik.

- `⌨️ Pintasan Global`: `Enter` berputar melalui enam preset (`Alt+Space`, `Ctrl+Space`, `Ctrl+Shift+Space`, `Ctrl+Alt+Space`, `Alt+Shift+Space`, `Ctrl+Shift+Y`); ini bukan mode merekam tombol. Saat preset diganti, jika kombinasi yang dipilih sibuk Yasal mencoba preset berikutnya.
- `🚀 Mulai bersama Windows`: tambah/hapus nilai `Yasal` pada key registry pengguna `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`; nilainya menunjuk ke executable saat ini.
- `🎨 Tema Tampilan`: rotasi `Otomatis (Sistem)` → `Gelap` → `Terang`.
- Toggle plugin: Kalkulator, Pencarian Web, Perintah Shell, Perintah Sistem, Pencarian File, Pengaturan Windows, Riwayat Clipboard, Informasi Jaringan, dan Pengalih Jendela. Perubahan diterapkan tanpa memulai ulang.
- `🌐 Mesin Pencari Bawaan`: rotasi Google, DuckDuckGo, dan Bing.
- `📁 Folder Pencarian File`: `Enter` membuka dialog pilih folder (maksimal 8). Setiap baris folder bertuliskan `[HAPUS]`; `Enter` pada baris itu menghapus folder dari daftar. Daftar kosong berarti cakupan default.
- `ℹ️ Tentang Yasal`: menampilkan versi `0.1.0`; baris ini tidak interaktif.

---

## 8. Theming

- **Integrasi tema Windows**:
  - Mode aplikasi dibaca dari nilai registry `AppsUseLightTheme`; diperiksa ulang sekitar setiap 5 detik saat palette terlihat.
  - Pilihan manual di Settings: `Otomatis (Sistem)`, `Gelap`, atau `Terang`.
- **Accent Color**: diambil melalui `DwmGetColorizationColor` dan dipakai pada tema egui.
- **Transparansi/material**: window transparan menggunakan permukaan dan frame kartu solid yang digambar egui; Acrylic/Mica blur belum diimplementasikan.

---

## 9. Window Behavior

| Properti | Nilai |
|---|---|
| Posisi | Monitor di bawah kursor; fallback ke monitor utama bila pembacaan gagal. Kartu dipusatkan horizontal dan ditempatkan menurut tinggi maksimum; DPI diambil dari monitor tujuan |
| Sudut & border | Radius 26px; border aksen selebar 1px digambar egui; sudut dan border DWM sistem dinonaktifkan |
| Shadow | Dilukis egui dalam margin transparan 18px di luar kartu |
| Margin konten | Horizontal 16px, vertikal 8px |
| Animasi | Fade-in 100ms; fade-out belum ada, window langsung disembunyikan |

---

## 10. Ringkasan Keputusan (Quick Reference)
✅ Hasil seragam: baris 44px, gap 4px, maksimal 8 baris terlihat sebelum scroll, dan badge kategori.
✅ Ikon shell 64×64 diekstrak secara lazy dan dirender pada area 28×28 dengan cache PNG di disk.
✅ Ikon mode, input baru saat palette dibuka, dan `Esc` membersihkan query lalu menutup.
✅ Empty state berbasis frecency; web search berperingkat rendah sebagai fallback global.
✅ Settings dapat dicari, dipilih dengan keyboard, serta diaktifkan lewat `Enter` atau klik.
✅ Action Panel berisi aksi sesuai jenis item; status bar menunjukkan petunjuk per kategori.
✅ Tema sistem/manual, accent color DWM, dan penempatan monitor mengikuti posisi kursor serta DPI.
✅ Pengosongan Recycle Bin tanpa dialog; konfirmasi native untuk shutdown/restart/sign out.

Material Acrylic/Mica, color picker, installer, dan auto-updater belum tersedia.

---

## 11. Roadmap

### Belum diimplementasikan

- **Color Picker**: overlay pemilih warna layar, loupe pembesar, dan salin kode warna.
- **Installer/distribusi paket**: installer WiX/MSIX atau paket distribusi siap pasang; README menjelaskan instalasi manual.
- **Auto-updater**.

### Sudah tersedia

- Pencarian file melalui `es.exe` dengan fallback indeks lokal terbatas.
