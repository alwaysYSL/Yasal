# 🌸 Yasal (يسأل)

<div align="center">

**Windows Command Palette berbasis Rust.**

*Cari aplikasi, file, dan perintah dengan keyboard.*

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078d7.svg)](https://microsoft.com/windows)

</div>

---

## ✨ Fitur inti

- ⚡ **Global Hotkey (`Alt + Space`)**: Akses instan di mana saja di seluruh Windows.
- 🚀 **Peluncur Aplikasi**: mencari shortcut Start Menu/Desktop dan aplikasi Windows yang terdaftar di AppsFolder.
  - `Enter`: Buka aplikasi.
  - `Ctrl + Enter`: Jalankan shortcut aplikasi sebagai Administrator bila tersedia; aplikasi AppsFolder tanpa aksi sekunder dibuka seperti biasa.
  - `Tab`: Buka Action Panel jika item menyediakan aksi tambahan.
- 🔢 **Kalkulator Cepat**:
  - Ketik ekspresi matematika (`= 25 * 4 + 10` atau `100 / 4`), tekan `Enter` untuk langsung menyalin hasil ke clipboard.
  - Pembagian memakai bilangan real (`10/4` = `2.5`) dan hasilnya dibulatkan tanpa sisa floating point (`0.1+0.2` = `0.3`). Gunakan `.` sebagai pemisah desimal; fungsi seperti `= math::sqrt(16)` juga didukung.
  - **Konversi satuan** tanpa internet: `5 km to mi`, `72 f in c`, `1.5 jam ke menit`, `2 gb to mb`. Mendukung panjang, massa, volume, suhu, waktu, data, luas, dan kecepatan dengan nama Inggris maupun Indonesia. `Enter` menyalin angkanya saja, `Ctrl + Enter` menyalin beserta satuan. Data memakai awalan SI (1 KB = 1000 B); gunakan `KiB`, `MiB`, `GiB` untuk basis 1024.
- 🌐 **Pencarian Web & URL Handler**:
  - Prefix `??` untuk mencari di Google/DuckDuckGo/Bing, atau ketik langsung tautan web (`https://...`).
  - **Alias**: tambahkan entri `web_aliases` di `config.json`, misalnya `{"keyword": "yt", "url": "https://www.youtube.com/results?search_query={query}"}`, lalu ketik `yt rust tutorial`. Keyword hanya boleh berisi huruf, angka, `-`, dan `_` (maksimal 16 karakter). URL harus berawalan `http://` atau `https://` dan memuat `{query}`. Keyword `file`, `cb`, `ip`, `network`, `w`, dan `settings` tidak dapat dipakai. Restart Yasal setelah mengubah `config.json`.
- 💻 **Perintah Shell**:
  - Prefix `>` untuk menjalankan perintah langsung di Windows PowerShell.
  - `Enter` membuka jendela PowerShell baru. `Ctrl + Enter` menjalankan perintah di latar belakang dengan batas 15 detik dan menampilkan hingga 200 baris output di palette. `Enter` pada baris output menyalin baris itu, dan `Enter` pada baris ringkasan menyalin seluruh output.
- 🛡️ **Perintah Sistem Windows**: kunci layar, Sleep, kosongkan Recycle Bin, Shutdown, Restart, dan Sign Out. Shutdown, Restart, dan Sign Out meminta konfirmasi; pengosongan Recycle Bin berjalan tanpa dialog konfirmasi.
- ⚙️ **Built-in Settings View**: ketik `settings` untuk membuka dan memfilter pengaturan. Navigasi keyboard tersedia; baris pengaturan juga dapat diklik.
- 🎨 **Tema Windows**: tema mengikuti mode aplikasi Windows dan warna aksen DWM. Saat palette terlihat, mode sistem diperiksa berkala; tema Gelap/Terang juga tersedia di Settings.
- 🧠 **Memory Trimming Otomatis**:
  - Saat palette disembunyikan, Yasal meminta Windows mengurangi **working set** lewat `EmptyWorkingSet`. Nilai memori aktual bergantung pada sistem; working set tidak sama dengan Private Bytes.

---

## 🔎 Pencarian dan fitur tambahan

- **Pencarian file**: gunakan `file <query>` untuk mencari secara eksplisit. Untuk query global minimal 3 karakter yang bukan URL, Yasal juga mencoba pencarian file bila tidak ada kecocokan lokal selain fallback Web. Yasal mencari `es.exe` di `PATH` dan lokasi instalasi Everything umum; tanpa `es.exe` atau jika pencarian Everything gagal, pencarian pertama membangun indeks rekursif dari Documents, Desktop, dan Downloads, dibatasi hingga 15.000 entri dan 4 MiB path. Folder pencarian dapat diatur di `settings` lewat baris **Folder Pencarian File**: `Enter` membuka dialog pilih folder Windows, dan `Enter` pada baris folder menghapusnya (maksimal 8 folder). Jika daftar kosong, Everything mencari seluruh indeksnya, atau tanpa Everything, pencarian memakai Documents, Desktop, dan Downloads. Jika daftar berisi folder, Everything hanya mencari di folder tersebut (`es.exe -path`), dan indeks fallback hanya membaca folder itu. Pencarian berjalan di background dengan debounce 150 ms.
- **Pengalih jendela**: gunakan `w` untuk melihat jendela terbuka atau `w <teks>` untuk memfilter judul/nama program; `Enter` beralih dan memulihkan jendela yang diminimalkan. `Tab` menawarkan aksi menutup jendela.
- **Pencarian Windows Settings**: gunakan `$ <query>`, misalnya `$ bluetooth`, `$ display`, atau `$ update`.
- **Riwayat clipboard**: gunakan `cb` atau `cb <query>` untuk menemukan teks yang disalin. Hingga 50 item disimpan sementara di memori saat plugin aktif; riwayat dibersihkan saat plugin atau aplikasi ditutup.
- **Informasi jaringan**: gunakan `ip` atau `network` untuk melihat IPv4 lokal, gateway, alamat MAC, dan IP publik. Pengambilan IP publik meminta `https://api.ipify.org` di background dan menyimpan hasil 60 detik.
- **Settings**: ketik `settings` untuk memfilter dan mengubah pengaturan, termasuk plugin, tema, hotkey, dan mesin pencari. Plugin dapat diubah tanpa memulai ulang.
- **Posisi palette**: monitor di bawah kursor mouse; fallback ke monitor utama bila pembacaan monitor gagal.

Tema mengikuti mode aplikasi Windows dan warna aksen DWM; mode sistem diperiksa berkala saat palette terlihat. Tema Gelap/Terang juga tersedia di Settings.

---

## ⌨️ Pintasan Keyboard

| Tombol | Aksi |
|---|---|
| `Alt + Space` | Buka / Sembunyikan palette Yasal |
| `↑` / `↓` | Navigasi hasil atau daftar aksi |
| `Enter` | Jalankan aksi utama; pada kalkulator dan beberapa hasil jaringan/clipboard, salin hasil atau nilai |
| `Ctrl + Enter` | Jalankan aksi sekunder bila tersedia, misalnya Run as Admin, tampilkan file di Explorer, salin hasil, atau jalankan perintah `>` dan tampilkan outputnya |
| `Tab` | Buka **Action Panel** bila item terpilih menyediakan aksi tambahan |
| `Esc` | Di Action Panel atau Settings: kembali ke pencarian · Di pencarian: bersihkan teks jika ada, jika kosong tutup palette |

---

## Build, instal, dan jalankan

### Persyaratan

- Windows 10 versi 1809 atau Windows 11.
- Rust toolchain, target `x86_64-pc-windows-gnu`, dan linker MinGW-w64 yang kompatibel untuk build dari source. Target build ditetapkan di `.cargo/config.toml`.

### Build dan tes

Jalankan dari root repository. Pasang target Rust jika belum tersedia:

```powershell
rustup target add x86_64-pc-windows-gnu
```

Build/run:

```powershell
cargo run
cargo test
cargo build --release
```

`cargo run` menjalankan build development. `cargo test` menjalankan unit dan integration tests. Biner release berada di `target/x86_64-pc-windows-gnu/release/yasal.exe`.

### Instalasi manual

Repository belum menyediakan installer atau paket rilis. Setelah build release, jalankan perintah berikut dari root repository untuk memasang biner di lokasi tetap:

```powershell
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\Yasal'
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item 'target\x86_64-pc-windows-gnu\release\yasal.exe' (Join-Path $installDir 'yasal.exe')
Start-Process -FilePath (Join-Path $installDir 'yasal.exe')
```

Yasal normalnya berjalan di background dengan ikon system tray; tekan `Alt + Space` untuk membuka palette. Jika ingin Yasal berjalan bersama Windows, aktifkan `Mulai bersama Windows` di `settings` setelah biner diletakkan di lokasi tetap. Windows menyimpan path executable saat itu di registry; nonaktifkan startup sebelum memindahkan biner.

### Konfigurasi dan data lokal

| Lokasi Windows | Isi |
|---|---|
| `%APPDATA%\Yasal\config.json` | Preferensi aplikasi dan plugin |
| `%APPDATA%\Yasal\frecency.json` | Riwayat penggunaan untuk ranking hasil |
| `%LOCALAPPDATA%\Yasal\IconCache\` | Cache ikon aplikasi/file |
| `%LOCALAPPDATA%\Yasal\yasal_debug.log` | Log diagnostik, dibatasi sekitar 512 KiB sebelum dimulai ulang |

Riwayat clipboard hanya berada di memori selama plugin aktif; tidak disimpan atau dipulihkan setelah aplikasi dimulai ulang. Jika direktori data lokal Windows tidak tersedia, cache ikon/log menggunakan direktori kerja aplikasi. Jika direktori konfigurasi tidak tersedia, `config.json` menggunakan `Yasal\config.json` di direktori kerja; frecency tidak disimpan tanpa direktori konfigurasi.

### Menghapus instalasi

1. Jika fitur `Mulai bersama Windows` aktif, buka Yasal terlebih dahulu dan matikan pengaturan itu di `settings`. Bila proses Yasal sudah dihentikan, jalankan kembali executable untuk mematikannya.
2. Pilih **Exit** dari menu ikon system tray.
3. Hapus folder tempat `yasal.exe` disimpan.
4. Opsional, hapus `%APPDATA%\Yasal` dan `%LOCALAPPDATA%\Yasal` untuk menghapus preferensi, frecency, cache ikon, dan log.

Paket installer dan auto-updater belum tersedia; pemasangan, pembaruan, dan penghapusan saat ini dilakukan manual.

---

## 📄 Lisensi dan distribusi

Source tree belum menyertakan berkas `LICENSE`; klaim lisensi MIT belum dapat diverifikasi. Jangan mendistribusikan aplikasi sebagai MIT sebelum pemegang hak cipta menetapkan dan menambahkan lisensi yang sesuai.
