# 🏗️ ChaCrab: Technical Architecture

Dokumen ini menjelaskan desain sistem, model keamanan, dan skema data yang digunakan dalam ChaCrab.

---

## 🛡️ Security Model (Zero-Knowledge)

ChaCrab menggunakan prinsip **Zero-Knowledge Architecture**. Artinya, server (Supabase) tidak pernah mengetahui _Master Password_ pengguna atau isi data yang disimpan.

### 1. Key Derivation (Argon2id)

Sebelum enkripsi dilakukan, _Master Password_ diubah menjadi kunci kriptografi yang kuat.

- **Input**: Master Password + Unique User Salt
- **Algorithm**: Argon2id (Konfigurasi: Memory 64MB, Iterations 3, Parallelism 4)
- **Output**: 32-byte _Derived Key_

### 2. Authenticated Encryption (ChaCha20-Poly1305)

Kami menggunakan _Stream Cipher_ modern yang sangat cepat dan aman.

- Setiap entri data menggunakan **Nonce** (12-byte) unik yang dihasilkan secara acak melalui `OsRng`
- Data yang dienkripsi mencakup: Username, Password, dan Isi Catatan Rahasia

---
    

## 🔄 Data Flow: Create New Credential

1. **User Input**: Pengguna memasukkan label, username, dan password via CLI

2. **Encryption**:
   - Sistem mengambil _Derived Key_ dari memory/OS Keyring
   - Menghasilkan _Nonce_ acak
   - Melakukan enkripsi lokal pada username & password

3. **Transmission**: Data terenkripsi dikirim ke Supabase melalui HTTPS

4. **Storage**: Supabase menyimpan _Ciphertext_ dan _Nonce_

---
    

## 🗄️ Database Schema

ChaCrab menggunakan PostgreSQL di atas platform Supabase. Berikut adalah struktur tabel utamanya:

### 1. Table: `user_configs`

Menyimpan informasi dasar untuk proses login dan derivasi kunci.

| Column     | Type        | Description                        |
|------------|-------------|------------------------------------|
| user_id    | UUID (PK)   | Relasi ke auth.users Supabase      |
| salt       | TEXT        | Salt unik per user untuk Argon2id  |
| updated_at | TIMESTAMPTZ | Waktu perubahan terakhir           |

### 2. Table: `credentials`

Menyimpan data akun yang terenkripsi.

| Column       | Type      | Description                          |
|--------------|-----------|--------------------------------------|
| id           | UUID (PK) | Unique ID entri                      |
| user_id      | UUID (FK) | Pemilik data (dengan RLS aktif)      |
| label        | TEXT      | Nama aplikasi/layanan (Plaintext)    |
| url          | TEXT      | URL layanan (Plaintext)              |
| enc_username | TEXT      | Username terenkripsi (Base64)        |
| enc_password | TEXT      | Password terenkripsi (Base64)        |
| nonce        | TEXT      | Nonce unik untuk baris ini (Base64)  |

### 3. Table: `secret_notes`

Menyimpan catatan teks panjang yang terenkripsi.

| Column      | Type      | Description                             |
|-------------|-----------|-----------------------------------------|
| id          | UUID (PK) | Unique ID entri                         |
| title       | TEXT      | Judul catatan (Plaintext)               |
| enc_content | TEXT      | Isi catatan terenkripsi (Base64)        |
| nonce       | TEXT      | Nonce unik untuk baris ini (Base64)     |
| tags        | TEXT[]    | Array label untuk pengelompokan (Plaintext) |

---

## 🔑 Session Management (OS Keyring)

Untuk menjaga kenyamanan tanpa mengorbankan keamanan, ChaCrab menggunakan **OS-level secure storage**:

- **macOS**: Keychain Access
- **Linux**: Secret Service (libsecret) atau KWallet
- **Windows**: Credential Manager

_Derived Key_ disimpan di sini setelah login berhasil dan dihapus secara otomatis saat pengguna menjalankan perintah logout.

---

## 📂 Project Structure

Struktur kode diatur secara modular untuk memudahkan pengujian unit:

```plaintext
src/
├── main.rs          # Handler argumen CLI (Clap)
├── crypto/          # Logika murni Argon2 & ChaCha20
├── storage/         # Implementasi SQLx dan koneksi Supabase
├── commands/        # Logika bisnis per perintah (Add, Get, Ls)
├── models/          # Struct data (Structs for DB rows)
└── ui/              # Formatting output terminal & prompts
```
