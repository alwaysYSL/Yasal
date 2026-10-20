# Perbaikan CPU Saat Idle

## Gejala dan pengukuran

Pada Windows dengan 12 logical processor, proses Yasal memakai sekitar 8% CPU ketika jendela tersembunyi. Uji biner release menunjukkan 0,982 core CPU sesudah 12 detik idle. Lonjakan juga terjadi setelah palette dibuka lalu ditutup: 0,977 core.

## Penyebab

`eframe` 0.29.1 menjadwalkan repaint dengan `ControlFlow::WaitUntil`. Ketika waktunya tiba, integrasi native mengubah event loop ke `ControlFlow::Poll` dan memanggil `window.request_redraw()`. Windows tidak mengirim event redraw untuk HWND yang disembunyikan menggunakan `ShowWindow(SW_HIDE)`, sehingga event loop terus melakukan polling. Timer 10 detik pada jalur tersembunyi memicu kasus pertama; timer dari frame yang terlihat dan dari widget `egui` dapat memicu kasus buka–tutup.

## Perbaikan

- `src/app.rs` tidak lagi menjadwalkan repaint 10 detik saat jendela tersembunyi. Frame juga berhenti setelah hotkey atau tray menutup palette.
- `vendor/eframe/src/native/run.rs` memeriksa `window.is_visible()` sebelum meminta redraw. Repaint yang jatuh tempo saat jendela tersembunyi dibuang dan event loop kembali ke `ControlFlow::Wait`.
- `Cargo.toml` memakai patch lokal `eframe` 0.29.1 agar perilaku ini ikut masuk ke build release. Jadwal repaint dan pemeriksaan fokus/tema saat palette terlihat tetap berlaku.

## Verifikasi

Jalankan dari root proyek pada Windows:

```powershell
cargo build --release
pwsh -NoProfile -File tests/idle_cpu.ps1 -Executable target/x86_64-pc-windows-gnu/release/yasal.exe
pwsh -NoProfile -File tests/idle_cpu.ps1 -Executable target/x86_64-pc-windows-gnu/release/yasal.exe -OpenClose
cargo test
```

Uji CPU menjalankan biner sebenarnya, menunggu lewat dari timer lama, dan mengukur CPU selama empat detik. Mode `-OpenClose` mengirim `WM_HOTKEY` ke listener milik proses uji, memeriksa bahwa palette terbuka dan tertutup, lalu memastikan palette dapat dibuka lagi setelah idle.

Hasil pada 24 September 2026:

| Keadaan | Sebelum | Sesudah |
|---|---:|---:|
| Tersembunyi sejak startup | 0,982 core (8,19%) | 0,004 core (0,03%) |
| Tersembunyi setelah buka–tutup | 0,977 core (8,14%) | 0,000 core (0,00%) |

Pengukuran ini hanya untuk CPU saat jendela tersembunyi. CPU ketika mengetik atau memuat hasil asinkron belum diukur di sini.
