# 🦀 ChaCrab Agent Context

Dokumen ini berfungsi sebagai instruksi utama bagi AI Agent dalam mengembangkan dan memelihara ChaCrab, pengelola rahasia berbasis CLI dengan prinsip Zero-Knowledge.

---

## 🎯 Core Principles (Strict)

- **Zero-Knowledge First**: Jangan pernah mengusulkan solusi yang mengirim Master Password atau Plaintext ke server. Enkripsi HARUS terjadi di sisi klien (`src/crypto/`).

- **Memory Safety**: Gunakan fitur ownership Rust secara maksimal. Hindari `unsafe` block kecuali sangat diperlukan untuk interaksi OS Keyring.

- **No Disk Persistence**: Jangan menulis kunci derivasi ke file `.txt` atau log. Gunakan `keyring-rs` untuk sesi.

- **Nonce Uniqueness**: Setiap enkripsi wajib menggunakan nonce baru yang unik (12-byte).

---

## 🛠 Tech Stack Reference

- **Language**: Rust (Latest Stable)
- **CLI**: clap v4 (Derive API)
- **Crypto**: chacha20poly1305, argon2
- **Database**: sqlx (PostgreSQL/Supabase)
- **Storage**: keyring crate untuk manajemen session key di level OS

---

## 📂 Architecture Map

Jika diminta menambahkan fitur, rujuk ke struktur berikut:

- **Logika Kripto**: Tambahkan ke `src/crypto/`. Jangan mencampur logika enkripsi di dalam `commands/`.

- **Query Database**: Gunakan makro `sqlx::query!` di dalam `src/storage/supabase.rs`.

- **UI/UX**: Gunakan `dialoguer` untuk input interaktif di `src/ui/`.

- **Schema**: Update `migrations/` jika ada perubahan struktur tabel `credentials` atau `secret_notes`.

---

## 📋 Task Guidelines

### 1. Menambah Perintah CLI Baru

- Definisikan sub-command di `src/main.rs` menggunakan clap.
- Buat modul baru di `src/commands/`.
- Pastikan fungsi mengembalikan `anyhow::Result<()>` untuk penanganan error yang bersih.

### 2. Penanganan Error

- Gunakan crate `anyhow` untuk aplikasi CLI.
- Berikan pesan error yang informatif kepada pengguna terminal (misal: "Koneksi Supabase gagal, periksa .env").

### 3. Keamanan Data

- Saat menampilkan password, gunakan fitur "copy to clipboard" atau sembunyikan input menggunakan `dialoguer::Password`.

---

## ⚠️ Common Pitfalls to Avoid

- **Hardcoding Key**: Jangan pernah menaruh Salt atau Key statis di dalam kode.

- **SQL Injection**: Selalu gunakan parameterized queries dari sqlx.

- **Dependencies**: Jangan menambah crate baru tanpa alasan yang kuat untuk menjaga binary tetap ringan.