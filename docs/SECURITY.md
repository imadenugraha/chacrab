# 🔐 ChaCrab Security Documentation

This document provides a deep dive into ChaCrab's security architecture, threat model, and cryptographic implementation.

## Overview

ChaCrab implements a **Zero-Knowledge Architecture** where:
1. All encryption happens client-side
2. Master passwords are never stored
3. The database only contains ciphertext
4. Session keys are managed by the OS keyring

## Cryptographic Primitives

### 1. Key Derivation: Argon2id

**Purpose**: Convert the user's master password into a strong encryption key.

**Configuration**:
```
Algorithm: Argon2id (hybrid mode)
Memory Cost: 64 MB (65,536 KB)
Iterations: 3
Parallelism: 4 threads
Output Length: 32 bytes (256 bits)
Salt: Unique random salt per vault (generated at init)
```

**Why Argon2id?**
- Winner of the Password Hashing Competition (2015)
- Resistant to GPU/ASIC attacks via memory-hardness
- Hybrid mode combines data-dependent (Argon2i) and data-independent (Argon2d) approaches
- Protects against both side-channel and GPU attacks

**Parameter Rationale**:
- **64 MB memory**: Balances security and usability (higher is more secure but slower)
- **3 iterations**: Sufficient time cost without impacting user experience
- **4 threads**: Utilizes modern multi-core CPUs efficiently

### 2. Authenticated Encryption: ChaCha20-Poly1305

**Purpose**: Encrypt credential data with authentication to prevent tampering.

**Specifications**:
```
Cipher: ChaCha20 (stream cipher)
MAC: Poly1305 (message authentication code)
Key Length: 256 bits
Nonce Length: 96 bits (12 bytes)
Authentication Tag: 128 bits (16 bytes)
```

**Why ChaCha20-Poly1305?**
- Modern, fast, and secure alternative to AES-GCM
- Better performance on systems without hardware AES support
- Approved by IETF (RFC 8439)
- Used by major systems: TLS 1.3, SSH, WireGuard

**Security Properties**:
- **Confidentiality**: Ciphertext reveals nothing about plaintext
- **Authenticity**: Any modification to ciphertext is detected
- **Nonce Misuse Resistance**: Each encryption uses unique random nonce

### 3. Random Number Generation

**Source**: Operating System's cryptographically secure RNG (`OsRng`)

**Usage**:
- Generating unique nonces for each encryption operation
- Creating random salts during vault initialization

## Data Flow

### Initialization (`chacrab init`)

```
1. User enters master password
   ↓
2. Generate unique random salt (22 chars base64)
   ↓
3. Derive 256-bit key: Argon2id(password, salt)
   ↓
4. Store salt in database (plaintext)
   ↓
5. Store derived key in OS keyring (secure)
```

### Adding a Credential (`chacrab add`)

```
1. User enters username and password
   ↓
2. Retrieve derived key from OS keyring
   ↓
3. For each field (username, password):
   a. Generate unique 96-bit nonce
   b. Encrypt: ChaCha20-Poly1305(key, nonce, plaintext)
   c. Encode ciphertext and nonce as base64
   ↓
4. Store in database:
   - label (plaintext)
   - url (plaintext)
   - enc_username (base64 ciphertext)
   - enc_password (base64 ciphertext)
   - nonce_username (base64)
   - nonce_password (base64)
```

### Retrieving a Credential (`chacrab get`)

```
1. User specifies label
   ↓
2. Retrieve encrypted data from database
   ↓
3. Get derived key from OS keyring
   ↓
4. For each field:
   a. Decode base64 ciphertext and nonce
   b. Decrypt: ChaCha20-Poly1305_decrypt(key, nonce, ciphertext)
   c. Verify authentication tag
   ↓
5. Display or copy to clipboard
```

## Threat Model

### What ChaCrab Protects Against

✅ **Database Theft**
- Attacker gets `chacrab.db` file
- Cannot decrypt without master password
- Would need to brute-force Argon2 (computationally infeasible)

✅ **Network Interception**
- All encryption happens locally
- Nothing sensitive transmitted over network (local database)

✅ **Disk Access While Locked**
- Session key cleared from keyring on logout
- Database contains only ciphertext

✅ **Data Tampering**
- Poly1305 MAC detects any modification
- Decryption fails if ciphertext is altered

✅ **Password Reuse Detection**
- Each credential encrypted with unique nonce
- Even identical passwords produce different ciphertexts

### What ChaCrab Does NOT Protect Against

❌ **Compromised Operating System**
- Keyloggers can capture master password during entry
- Malware with root access can read from OS keyring
- Memory dumps can extract keys while logged in

❌ **Physical Access While Unlocked**
- Attacker with CLI access while logged in can retrieve credentials
- No additional authentication after login

❌ **Weak Master Passwords**
- Short or dictionary passwords are vulnerable to brute-force
- Minimum 8 characters enforced, but longer is strongly recommended

❌ **Master Password Loss**
- No recovery mechanism (by design)
- Lost master password = lost data

❌ **Shoulder Surfing**
- Displayed passwords visible on screen
- Use clipboard copy feature to mitigate

❌ **Malicious Dependencies**
- Supply chain attacks in Rust crates
- Mitigated by using well-audited cryptographic libraries

## Security Best Practices

### For Users

1. **Use a Strong Master Password**
   - Minimum: 16+ characters
   - Use a passphrase: "correct horse battery staple" style
   - Never reuse from other services
   - Consider using dice-generated passphrases

2. **Secure Your System**
   - Keep OS and software updated
   - Use full-disk encryption
   - Enable firewall and antivirus
   - Lock screen when away

3. **Regular Backups**
   - Backup `chacrab.db` frequently
   - Store backups encrypted
   - Test restoration process

4. **Logout When Done**
   - Run `chacrab logout` after use
   - Clears session key from memory/keyring

5. **Audit Stored Credentials**
   - Review `chacrab list` periodically
   - Remove unused credentials
   - Update compromised passwords

### For Developers

1. **Code Review**
   - Scrutinize all cryptographic code
   - Verify correct use of primitives
   - Check for timing attacks

2. **Dependency Management**
   - Pin cryptographic library versions
   - Review dependency updates carefully
   - Use `cargo audit` for known vulnerabilities

3. **Testing**
   - Maintain comprehensive test coverage
   - Test error conditions
   - Fuzz cryptographic functions

4. **Secure Defaults**
   - Never log sensitive data
   - Clear sensitive data from memory when possible
   - Use secure random number generation

## Cryptographic Assumptions

ChaCrab's security relies on:

1. **Argon2id** is resistant to pre-image attacks
2. **ChaCha20** is a secure stream cipher
3. **Poly1305** is a secure MAC
4. **OS RNG** provides cryptographically secure randomness
5. **OS Keyring** securely stores session keys
6. Attacker cannot bypass OS security (no kernel exploits)

## Known Limitations

1. **Memory Safety**
   - Master password exists in memory briefly during login
   - Rust's ownership helps but doesn't guarantee memory isn't swapped to disk
   - Future: Use memory locking (mlock) for sensitive data

2. **Nonce Reuse**
   - Critical: Never encrypt with same (key, nonce) pair
   - Mitigated: Each encryption generates random nonce
   - Risk: If OS RNG is compromised, nonce collisions possible

3. **Timing Attacks**
   - Password comparison could leak information via timing
   - Mitigated: Argon2 includes constant-time operations
   - Database queries may have timing variability (low risk)

4. **Side Channels**
   - Cache timing attacks possible on CPU
   - Mitigated: ChaCha20 is constant-time
   - Electromagnetic emanations not addressed

## Audit History

**Status**: Not yet audited

ChaCrab uses industry-standard, audited cryptographic libraries:
- `chacha20poly1305` crate: RustCrypto implementation
- `argon2` crate: Official Rust PHC implementation

However, the application code itself has not undergone professional security audit. Community audits and contributions are welcome.

## Incident Response

If a vulnerability is discovered:

1. **Do NOT** disclose publicly
2. Email: security@example.com
3. Provide: description, reproduction steps, impact assessment
4. Allow 90 days for patch before disclosure

## References

- [Argon2 Specification](https://github.com/P-H-C/phc-winner-argon2/blob/master/argon2-specs.pdf)
- [RFC 8439: ChaCha20-Poly1305](https://www.rfc-editor.org/rfc/rfc8439.html)
- [RustCrypto Documentation](https://github.com/RustCrypto)
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)

## Conclusion

ChaCrab implements cryptographic best practices for local password storage. While no system is perfectly secure, the combination of Argon2id and ChaCha20-Poly1305 provides strong protection against common attack vectors.

**Remember**: The weakest link is usually the master password. Choose it wisely and guard it carefully.
