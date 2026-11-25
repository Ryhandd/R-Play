# R-Play (`rplay`)

Pemutar musik offline berbasis Terminal User Interface (TUI) yang ringan menggunakan Rust.

## Fitur

- Memindai folder dan subfolder untuk format **MP3, WAV, OGG, dan FLAC**
- Membaca metadata audio (Title, Artist, Album, Duration) menggunakan `lofty`
- Kontrol pemutaran penuh via keyboard (Play/Pause, Next, Previous, Volume)
- Mode **Shuffle**, **Repeat Off/All/One**, progress bar, dan pencarian playlist
- Konfigurasi folder musik default lokal (`--set-default`)
- Mendukung Windows dan Linux

## Instalasi

Pastikan Rust toolchain sudah terpasang, lalu build dari source:

```bash
cargo build --release
```

Binary hasil build berada di `target/release/rplay` (atau `target/release/rplay.exe` di Windows).

> **Catatan Linux:** Diperlukan paket `libasound2-dev` dan `pkg-config` saat kompilasi.

## Cara Menjalankan

Linux/macOS:
```bash
rplay ~/Music
```

Windows:
```powershell
rplay "D:\Music"
```

Opsi tambahan:
- `--shuffle`: memulai pemutaran secara acak
- `--repeat`: mengaktifkan pengulangan playlist (Repeat All)
- `--set-default`: menyimpan folder yang diberikan sebagai folder default, sehingga berikutnya cukup menjalankan `rplay`

## Shortcut Keyboard

| Tombol | Fungsi |
| --- | --- |
| `Space` | Play / Pause |
| `Enter` | Putar lagu yang dipilih |
| `↑` / `↓` | Navigasi playlist |
| `N` / `P` | Lagu berikutnya / sebelumnya |
| `+` / `-` | Naikkan / turunkan volume |
| `S` | Toggle Shuffle |
| `R` | Ganti mode Repeat (Off / All / One) |
| `/` | Cari lagu dalam playlist |
| `Q` | Keluar |

## Mendapatkan Binary dari GitHub Actions

1. Buka tab **Actions** di repository GitHub.
2. Pilih workflow **Build R-Play** terbaru yang berstatus sukses.
3. Unduh artifact sesuai sistem operasi Anda (`rplay-windows-x64` atau `rplay-linux-x64`).
