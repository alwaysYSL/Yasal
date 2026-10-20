# 🌸 Yasal (يسأل)

<div align="center">

**Lightweight Windows Command Palette**

*Ultra-ringan · Cepat · Terintegrasi Windows Native · Target RAM < 25 MB (< 5 MB saat idle)*

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078d7.svg)](https://microsoft.com/windows)

</div>

---

## ✨ Fitur Utama (Phase 1 MVP)

- ⚡ **Global Hotkey (`Alt + Space`)**: Akses instan di mana saja di seluruh Windows.
- 🚀 **Peluncur Aplikasi (App Launcher)**: Mendeteksi aplikasi Start Menu & Desktop secara otomatis dengan fuzzy matching super cepat.
  - `Enter`: Buka aplikasi
  - `Ctrl + Enter`: Jalankan sebagai Administrator (*Run as Admin*)
  - `Tab`: Buka *Action Panel* kontekstual (Salin path, buka folder lokasi di Explorer).
- 🔢 **Kalkulator Cepat**:
  - Ketik ekspresi matematika (`= 25 * 4 + 10` atau `100 / 4`), tekan `Enter` untuk langsung menyalin hasil ke clipboard.
- 🌐 **Pencarian Web & URL Handler**:
  - Prefix `??` untuk mencari di Google/DuckDuckGo/Bing, atau ketik langsung tautan web (`https://...`).
- 💻 **Perintah Shell**:
  - Prefix `>` untuk menjalankan perintah langsung di Windows PowerShell.
- 🛡️ **Perintah Sistem Windows**:
  - Kunci Layar (*Lock*), Mode Tidur (*Sleep*), Kosongkan Recycle Bin, Matikan Komputer (*Shutdown*), Mulai Ulang (*Restart*), dan Keluar Akun (*Sign Out*) disertai konfirmasi native.
- ⚙️ **Built-in Settings View**:
  - Ketik `settings` di search bar untuk membuka menu pengaturan interaktif dengan baris 44px dan jarak 4px tanpa mouse.
- 🎨 **Sinkronisasi Tema Windows**:
  - Otomatis mengikuti Dark/Light mode Windows dan Accent Color DWM aktif secara real-time.
- 🧠 **Memory Trimming Otomatis**:
  - Memanfaatkan `EmptyWorkingSet` Win32 API saat window di-hide sehingga konsumsi RAM turun menjadi **< 5 MB**.

---

## 🔎 Fitur Phase 2

- **Pencarian file**: gunakan `file <query>`, atau ketik nama file saat tidak ada hasil lokal lain. Yasal memakai Everything `es.exe` jika tersedia; jika tidak, Documents, Desktop, dan Downloads diindeks saat pencarian pertama. Pencarian berjalan di background dengan debounce 150 ms.
- **Pengalih jendela**: gunakan `w` untuk melihat semua jendela terbuka (yang terakhir dipakai di atas), atau `w <teks>` untuk memfilter berdasarkan judul atau nama program. `Enter` berpindah ke jendela itu dan memulihkannya jika diminimalkan; `Tab` membuka aksi `Tutup jendela`.
- **Posisi jendela**: palette muncul di monitor tempat kursor mouse berada, di tengah secara horizontal dan sedikit di atas tengah secara vertikal, dengan tepi atas yang selalu sama berapa pun jumlah hasilnya.
- **Navigasi Pengaturan Windows**: gunakan `$ <query>`, misalnya `$ bluetooth`, `$ display`, atau `$ update`.
- **Riwayat clipboard**: gunakan `cb` atau `cb <query>`. Enter menyalin item terpilih kembali ke clipboard. Maksimal 50 teks terbaru disimpan hanya di memori; listener berhenti dan riwayat dibersihkan saat plugin dimatikan.
- **Informasi jaringan**: gunakan `ip` atau `network` untuk IPv4 lokal, gateway, alamat MAC, dan IP publik.
  - Pengambilan IP publik mengirim request background ke `https://api.ipify.org` saat query pertama, lalu menyimpan hasil hingga 60 detik.
- Semua plugin Phase 2 dapat diaktifkan atau dinonaktifkan langsung melalui view `settings`. Pintasan global memiliki enam pilihan (tekan `Enter` pada barisnya untuk berganti) dan langsung aktif tanpa memulai ulang Yasal. Jika kombinasinya dipakai aplikasi lain, pilihan berikutnya dicoba otomatis.

---

## ⌨️ Pintasan Keyboard

| Tombol | Aksi |
|---|---|
| `Alt + Space` | Buka / Sembunyikan palette Yasal |
| `↑` / `↓` | Navigasi hasil atau daftar aksi |
| `Enter` | Eksekusi aksi utama / Salin hasil kalkulator |
| `Ctrl + Enter` | Aksi sekunder default (*Run as Admin*, salin path, dll.) |
| `Tab` | Buka **Action Panel** (in-place replacement) untuk item terpilih |
| `Esc` | Tekan 1x: Bersihkan teks input · Tekan 2x: Tutup palette |

---

## 🛠️ Kompilasi & Menjalankan

### Persyaratan
- [Rust Toolchain (2021 Edition)](https://www.rust-lang.org/)
- Windows 10 (1809+) atau Windows 11

### Menjalankan Mode Development
```bash
cargo run
```

### Menjalankan Unit & Integration Tests
```bash
cargo test
```

### Build Biner Release Teroptimasi
```bash
cargo build --release
```
Biner berekstensi `.exe` mandiri tersedia di `target/x86_64-pc-windows-gnu/release/yasal.exe`.

---

## 📄 Lisensi
Didistribusikan di bawah Lisensi MIT.
