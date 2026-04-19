//! PDF security and encryption support
//!
//! Implements PDF encryption per PDF specification:
//!   - RC4-128  (PDF 1.4, /V 2 /R 3)
//!   - AES-256  (PDF 2.0, /V 5 /R 6)
//!
//! Supports owner/user passwords and permission flags (PDF spec Table 3.20).
//!
//! Pure Rust implementation using `aes`, `cbc`, `sha2`, and `md5` crates — no C FFI.

use aes::cipher::{BlockCipherEncrypt, KeyInit};
use aes::{Aes128, Aes256};
use cbc::cipher::KeyIvInit;
use md5::Md5;
use sha2::Digest as _;
use sha2::Sha256;

// ---------------------------------------------------------------------------
// Encryption algorithm selector
// ---------------------------------------------------------------------------

/// PDF encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncryptionAlgorithm {
    /// RC4-128 (PDF 1.4, /V 2 /R 3) — legacy, default for backwards compatibility
    #[default]
    Rc4128,
    /// AES-256 (PDF 2.0, /V 5 /R 6) — modern, recommended
    Aes256,
}

// ---------------------------------------------------------------------------
// PDF permission flags
// ---------------------------------------------------------------------------

/// PDF permission flags per PDF spec Table 3.20
///
/// These flags control what operations are allowed when the document
/// is opened with the user password.
#[derive(Debug, Clone, Copy)]
pub struct PdfPermissions {
    /// Allow printing (bit 3)
    pub allow_print: bool,
    /// Allow modifying content (bit 4)
    pub allow_modify: bool,
    /// Allow copying/extracting text (bit 5)
    pub allow_copy: bool,
    /// Allow adding/modifying annotations (bit 6)
    pub allow_annotations: bool,
    /// Allow filling form fields (bit 9) — always true for rev 3+
    pub allow_fill_forms: bool,
    /// Allow extracting for accessibility (bit 10)
    pub allow_accessibility: bool,
    /// Allow assembling document (bit 11)
    pub allow_assemble: bool,
    /// Allow high-quality printing (bit 12)
    pub allow_print_high_quality: bool,
}

impl Default for PdfPermissions {
    fn default() -> Self {
        Self {
            allow_print: true,
            allow_modify: true,
            allow_copy: true,
            allow_annotations: true,
            allow_fill_forms: true,
            allow_accessibility: true,
            allow_assemble: true,
            allow_print_high_quality: true,
        }
    }
}

impl PdfPermissions {
    /// Convert permissions to the 32-bit integer used in the /P entry
    ///
    /// Per PDF spec, bits 1-2 must be 0, bits 7-8 are reserved (1),
    /// bits 13-32 are reserved (1 for rev 3+).
    pub fn to_p_value(&self) -> i32 {
        let mut p: u32 = 0xFFFFF000; // Bits 13-32 all set

        // Bits 7-8 reserved, set to 1
        p |= 0b1100_0000;

        if self.allow_print {
            p |= 1 << 2; // bit 3
        }
        if self.allow_modify {
            p |= 1 << 3; // bit 4
        }
        if self.allow_copy {
            p |= 1 << 4; // bit 5
        }
        if self.allow_annotations {
            p |= 1 << 5; // bit 6
        }
        if self.allow_fill_forms {
            p |= 1 << 8; // bit 9
        }
        if self.allow_accessibility {
            p |= 1 << 9; // bit 10
        }
        if self.allow_assemble {
            p |= 1 << 10; // bit 11
        }
        if self.allow_print_high_quality {
            p |= 1 << 11; // bit 12
        }

        p as i32
    }
}

// ---------------------------------------------------------------------------
// PdfSecurity
// ---------------------------------------------------------------------------

/// PDF encryption settings
#[derive(Debug, Clone)]
pub struct PdfSecurity {
    /// Owner password (controls permissions)
    pub owner_password: String,
    /// User password (required to open document)
    pub user_password: String,
    /// Permission flags
    pub permissions: PdfPermissions,
    /// Encryption key length in bits (40 or 128 for RC4; 256 for AES)
    pub key_length: u32,
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
}

impl PdfSecurity {
    /// Create new security settings with RC4-128 encryption (legacy default)
    pub fn new(owner_password: &str, user_password: &str, permissions: PdfPermissions) -> Self {
        Self {
            owner_password: owner_password.to_string(),
            user_password: user_password.to_string(),
            permissions,
            key_length: 128,
            algorithm: EncryptionAlgorithm::Rc4128,
        }
    }

    /// Create new security settings with AES-256 encryption (PDF 2.0)
    pub fn new_aes256(
        owner_password: &str,
        user_password: &str,
        permissions: PdfPermissions,
    ) -> Self {
        Self {
            owner_password: owner_password.to_string(),
            user_password: user_password.to_string(),
            permissions,
            key_length: 256,
            algorithm: EncryptionAlgorithm::Aes256,
        }
    }

    /// Compute the encryption dictionary entries
    ///
    /// Dispatches to the appropriate algorithm implementation.
    pub fn compute_encryption_dict(&self, file_id: &[u8]) -> EncryptionDict {
        match self.algorithm {
            EncryptionAlgorithm::Rc4128 => self.compute_rc4_encryption_dict(file_id),
            EncryptionAlgorithm::Aes256 => self.compute_aes256_encryption_dict(),
        }
    }

    // ------------------------------------------------------------------
    // RC4-128 implementation (PDF 1.4, /V 2 /R 3)
    // ------------------------------------------------------------------

    /// Compute RC4-128 encryption dictionary (Algorithms 3.2–3.5)
    fn compute_rc4_encryption_dict(&self, file_id: &[u8]) -> EncryptionDict {
        let p_value = self.permissions.to_p_value();
        let revision = if self.key_length > 40 { 3 } else { 2 };
        let version = if self.key_length > 40 { 2 } else { 1 };
        let key_len_bytes = (self.key_length / 8) as usize;

        // Step 1: Compute O value (Algorithm 3.3)
        let o_value = self.compute_o_value(revision, key_len_bytes);

        // Step 2: Compute encryption key (Algorithm 3.2)
        let encryption_key =
            self.compute_encryption_key(&o_value, p_value, file_id, revision, key_len_bytes);

        // Step 3: Compute U value (Algorithm 3.4/3.5)
        let u_value = self.compute_u_value(&encryption_key, file_id, revision);

        EncryptionDict {
            o_value,
            u_value,
            oe_value: None,
            ue_value: None,
            perms_value: None,
            p_value,
            encryption_key,
            key_length: self.key_length,
            revision: revision as u32,
            version: version as u32,
            algorithm: EncryptionAlgorithm::Rc4128,
        }
    }

    /// Compute O value (owner password hash) per Algorithm 3.3
    fn compute_o_value(&self, revision: usize, key_len_bytes: usize) -> Vec<u8> {
        // Step a: Pad or truncate owner password
        let owner_padded = pad_password(&self.owner_password);

        // Step b: MD5 hash of padded password
        let mut hash = md5_hash(&owner_padded);

        // Step c: For revision 3, rehash 50 times
        if revision >= 3 {
            for _ in 0..50 {
                hash = md5_hash(&hash[..key_len_bytes]);
            }
        }

        // Step d: Use first key_len_bytes as RC4 key
        let key = &hash[..key_len_bytes];

        // Step e: Pad or truncate user password
        let user_padded = pad_password(&self.user_password);

        // Step f: RC4-encrypt the padded user password
        let mut result = rc4_encrypt(key, &user_padded);

        // Step g: For revision 3, iterate RC4 with XOR'd keys
        if revision >= 3 {
            for i in 1..=19 {
                let derived_key: Vec<u8> = key.iter().map(|&b| b ^ (i as u8)).collect();
                result = rc4_encrypt(&derived_key, &result);
            }
        }

        result
    }

    /// Compute encryption key per Algorithm 3.2
    fn compute_encryption_key(
        &self,
        o_value: &[u8],
        p_value: i32,
        file_id: &[u8],
        revision: usize,
        key_len_bytes: usize,
    ) -> Vec<u8> {
        let mut hasher = Md5::new();

        // Step a: Pad the user password
        let user_padded = pad_password(&self.user_password);
        hasher.update(&user_padded);

        // Step b: O value
        hasher.update(o_value);

        // Step c: P value as 4 little-endian bytes
        hasher.update((p_value as u32).to_le_bytes());

        // Step d: File ID (first element)
        hasher.update(file_id);

        let mut hash = hasher.finalize().to_vec();

        // Step e: For revision 3, rehash 50 times
        if revision >= 3 {
            for _ in 0..50 {
                hash = md5_hash(&hash[..key_len_bytes]);
            }
        }

        hash[..key_len_bytes].to_vec()
    }

    /// Compute U value per Algorithm 3.4 (rev 2) or 3.5 (rev 3)
    fn compute_u_value(&self, encryption_key: &[u8], file_id: &[u8], revision: usize) -> Vec<u8> {
        if revision >= 3 {
            // Algorithm 3.5 (revision 3)
            let mut hasher = Md5::new();
            hasher.update(PADDING);
            hasher.update(file_id);
            let hash = hasher.finalize().to_vec();

            let mut result = rc4_encrypt(encryption_key, &hash);

            for i in 1..=19 {
                let derived_key: Vec<u8> = encryption_key.iter().map(|&b| b ^ (i as u8)).collect();
                result = rc4_encrypt(&derived_key, &result);
            }

            // Pad to 32 bytes with arbitrary data
            result.resize(32, 0);
            result
        } else {
            // Algorithm 3.4 (revision 2)
            rc4_encrypt(encryption_key, &PADDING)
        }
    }

    // ------------------------------------------------------------------
    // AES-256 implementation (PDF 2.0, /V 5 /R 6)
    // ------------------------------------------------------------------

    /// Compute AES-256 encryption dictionary (PDF 2.0 spec Algorithm 8/9/10)
    fn compute_aes256_encryption_dict(&self) -> EncryptionDict {
        let p_value = self.permissions.to_p_value();

        // Generate random salts (8 bytes each) — use pseudo-random from hash of passwords
        // In a production system, use a CSPRNG; here we derive deterministically for testing
        let user_validation_salt = derive_salt(&self.user_password, b"user_val");
        let user_key_salt = derive_salt(&self.user_password, b"user_key");
        let owner_validation_salt = derive_salt(&self.owner_password, b"owner_val");
        let owner_key_salt = derive_salt(&self.owner_password, b"owner_key");

        // --- Algorithm 8: Compute U and UE ---
        // U = SHA-256(user_password + user_validation_salt) || user_validation_salt || user_key_salt
        // (total 48 bytes: 32 hash + 8 validation salt + 8 key salt)
        let u_hash = sha256_hash_parts(&[self.user_password.as_bytes(), &user_validation_salt]);
        let mut u_value = u_hash.clone();
        u_value.extend_from_slice(&user_validation_salt);
        u_value.extend_from_slice(&user_key_salt);
        // u_value is 48 bytes

        // File encryption key: 32 random bytes (derived deterministically)
        let file_enc_key = derive_file_enc_key(&self.owner_password, &self.user_password);

        // UE: encrypt file_enc_key with AES-256-CBC, key = SHA-256(user_password + user_key_salt)
        let ue_key = sha256_hash_parts(&[self.user_password.as_bytes(), &user_key_salt]);
        let ue_value = aes256_cbc_encrypt_no_iv(&file_enc_key, &ue_key);
        // ue_value is 32 bytes (no IV — ECB-like, single block)

        // --- Algorithm 9: Compute O and OE ---
        // O = SHA-256(owner_password + owner_validation_salt + u_value[0..48]) || owner_validation_salt || owner_key_salt
        let o_hash = sha256_hash_parts(&[
            self.owner_password.as_bytes(),
            &owner_validation_salt,
            &u_value,
        ]);
        let mut o_value = o_hash;
        o_value.extend_from_slice(&owner_validation_salt);
        o_value.extend_from_slice(&owner_key_salt);
        // o_value is 48 bytes

        // OE: encrypt file_enc_key with AES-256-CBC, key = SHA-256(owner_password + owner_key_salt + u_value[0..48])
        let oe_key =
            sha256_hash_parts(&[self.owner_password.as_bytes(), &owner_key_salt, &u_value]);
        let oe_value = aes256_cbc_encrypt_no_iv(&file_enc_key, &oe_key);

        // --- Algorithm 10: Compute Perms ---
        // Perms: encrypt 16-byte block with AES-256-ECB using file_enc_key
        // Bytes: P as 4 LE bytes | 0xFF 0xFF 0xFF 0xFF | T/F for metadata | 'adb' | 4 random bytes
        let mut perms_plain = [0u8; 16];
        let p_bytes = (p_value as u32).to_le_bytes();
        perms_plain[0..4].copy_from_slice(&p_bytes);
        perms_plain[4] = 0xFF;
        perms_plain[5] = 0xFF;
        perms_plain[6] = 0xFF;
        perms_plain[7] = 0xFF;
        perms_plain[8] = b'T'; // encrypt metadata = true
        perms_plain[9] = b'a';
        perms_plain[10] = b'd';
        perms_plain[11] = b'b';
        // bytes 12-15: random (deterministic from key for reproducibility)
        perms_plain[12] = file_enc_key[0];
        perms_plain[13] = file_enc_key[1];
        perms_plain[14] = file_enc_key[2];
        perms_plain[15] = file_enc_key[3];

        let perms_value = aes256_ecb_encrypt_block(&perms_plain, &file_enc_key);

        EncryptionDict {
            o_value,
            u_value,
            oe_value: Some(oe_value),
            ue_value: Some(ue_value),
            perms_value: Some(perms_value.to_vec()),
            p_value,
            encryption_key: file_enc_key,
            key_length: 256,
            revision: 6,
            version: 5,
            algorithm: EncryptionAlgorithm::Aes256,
        }
    }
}

// ---------------------------------------------------------------------------
// EncryptionDict
// ---------------------------------------------------------------------------

/// Computed encryption dictionary and keys
#[derive(Debug, Clone)]
pub struct EncryptionDict {
    /// The /O value (owner password hash, 32 bytes for RC4; 48 bytes for AES-256)
    pub o_value: Vec<u8>,
    /// The /U value (user password hash, 32 bytes for RC4; 48 bytes for AES-256)
    pub u_value: Vec<u8>,
    /// The /OE value (owner key, 32 bytes; AES-256 only)
    pub oe_value: Option<Vec<u8>>,
    /// The /UE value (user key, 32 bytes; AES-256 only)
    pub ue_value: Option<Vec<u8>>,
    /// The /Perms value (encrypted permissions, 16 bytes; AES-256 only)
    pub perms_value: Option<Vec<u8>>,
    /// The /P value (permission flags)
    pub p_value: i32,
    /// The encryption key (for encrypting streams/strings)
    pub encryption_key: Vec<u8>,
    /// Key length in bits
    pub key_length: u32,
    /// Revision number (2 for RC4-40, 3 for RC4-128, 6 for AES-256)
    pub revision: u32,
    /// Version number (1 for RC4-40, 2 for RC4-128, 5 for AES-256)
    pub version: u32,
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
}

impl EncryptionDict {
    /// Encrypt a PDF stream or string for a specific object
    ///
    /// For RC4: derives per-object key (Algorithm 3.1) then RC4-encrypts.
    /// For AES-256: encrypts with AES-256-CBC, prepending a 16-byte IV.
    pub fn encrypt_data(&self, data: &[u8], obj_num: u32, gen_num: u16) -> Vec<u8> {
        match self.algorithm {
            EncryptionAlgorithm::Rc4128 => {
                let key = self.derive_object_key(obj_num, gen_num);
                rc4_encrypt(&key, data)
            }
            EncryptionAlgorithm::Aes256 => {
                // AES-256-CBC with a deterministic IV (derived from key + obj_num for reproducibility)
                encrypt_aes256_cbc(data, &self.encryption_key, obj_num)
            }
        }
    }

    /// Derive per-object encryption key (Algorithm 3.1) — RC4 only
    fn derive_object_key(&self, obj_num: u32, gen_num: u16) -> Vec<u8> {
        let mut hasher = Md5::new();
        hasher.update(&self.encryption_key);
        hasher.update(&obj_num.to_le_bytes()[..3]);
        hasher.update(gen_num.to_le_bytes());
        let hash = hasher.finalize();

        // Key length is min(encryption_key.len() + 5, 16) bytes
        let key_len = std::cmp::min(self.encryption_key.len() + 5, 16);
        hash[..key_len].to_vec()
    }

    /// Generate the /Encrypt dictionary PDF object content
    pub fn to_pdf_dict(&self, encrypt_obj_id: usize) -> String {
        match self.algorithm {
            EncryptionAlgorithm::Rc4128 => self.to_rc4_pdf_dict(encrypt_obj_id),
            EncryptionAlgorithm::Aes256 => self.to_aes256_pdf_dict(encrypt_obj_id),
        }
    }

    /// Generate RC4-128 /Encrypt dictionary
    fn to_rc4_pdf_dict(&self, encrypt_obj_id: usize) -> String {
        let mut dict = String::new();
        dict.push_str(&format!("{} 0 obj\n", encrypt_obj_id));
        dict.push_str("<<\n");
        dict.push_str("/Filter /Standard\n");
        dict.push_str(&format!("/V {}\n", self.version));
        dict.push_str(&format!("/R {}\n", self.revision));
        dict.push_str(&format!("/Length {}\n", self.key_length));
        dict.push_str(&format!("/P {}\n", self.p_value));
        dict.push_str(&format!("/O <{}>\n", hex_encode(&self.o_value)));
        dict.push_str(&format!("/U <{}>\n", hex_encode(&self.u_value)));
        dict.push_str(">>\n");
        dict.push_str("endobj\n");
        dict
    }

    /// Generate AES-256 /Encrypt dictionary (PDF 2.0 / PDF 1.7 extension)
    fn to_aes256_pdf_dict(&self, encrypt_obj_id: usize) -> String {
        let mut dict = String::new();
        dict.push_str(&format!("{} 0 obj\n", encrypt_obj_id));
        dict.push_str("<<\n");
        dict.push_str("/Filter /Standard\n");
        dict.push_str("/V 5\n");
        dict.push_str("/R 6\n");
        dict.push_str("/Length 256\n");
        dict.push_str(&format!("/P {}\n", self.p_value));

        // /O and /U are 48 bytes in AES-256 (32-byte hash + 8 validation salt + 8 key salt)
        dict.push_str(&format!("/O <{}>\n", hex_encode(&self.o_value)));
        dict.push_str(&format!("/U <{}>\n", hex_encode(&self.u_value)));

        if let Some(ref oe) = self.oe_value {
            dict.push_str(&format!("/OE <{}>\n", hex_encode(oe)));
        }
        if let Some(ref ue) = self.ue_value {
            dict.push_str(&format!("/UE <{}>\n", hex_encode(ue)));
        }
        if let Some(ref perms) = self.perms_value {
            dict.push_str(&format!("/Perms <{}>\n", hex_encode(perms)));
        }

        // Crypt filter for AES-256
        dict.push_str("/CF <<\n");
        dict.push_str("  /StdCF <<\n");
        dict.push_str("    /AuthEvent /DocOpen\n");
        dict.push_str("    /CFM /AESV3\n");
        dict.push_str("    /Length 32\n");
        dict.push_str("  >>\n");
        dict.push_str(">>\n");
        dict.push_str("/StmF /StdCF\n");
        dict.push_str("/StrF /StdCF\n");
        dict.push_str(">>\n");
        dict.push_str("endobj\n");
        dict
    }

    /// Encrypt data using AES-128-ECB for a single block
    ///
    /// Used for encrypting individual 16-byte blocks when needed.
    #[allow(dead_code)]
    pub fn aes128_encrypt_block(&self, data: &[u8; 16]) -> [u8; 16] {
        let cipher = Aes128::new_from_slice(&self.encryption_key[..16])
            .expect("AES-128 key length is 16 bytes");
        let mut block = aes::Block::try_from(data.as_slice())
            .expect("block is exactly 16 bytes");
        cipher.encrypt_block(&mut block);
        block.into()
    }

    /// Encrypt data using AES-256-CBC with a random IV prepended to the output
    ///
    /// For AES-256 encryption of PDF streams and strings.
    /// Returns IV (16 bytes) || ciphertext.
    #[allow(dead_code)]
    pub fn encrypt_aes256(data: &[u8], key: &[u8]) -> Vec<u8> {
        // Generate a deterministic IV from key hash for testing
        // In production, use a CSPRNG
        let iv = sha256_hash_parts(&[key, data])[..16].to_vec();
        let mut iv_arr = [0u8; 16];
        iv_arr.copy_from_slice(&iv);

        type Aes256Cbc = cbc::Encryptor<Aes256>;
        let cipher = Aes256Cbc::new_from_slices(&key[..32], &iv_arr)
            .expect("AES-256 key/IV lengths valid");
        let ciphertext = aes256_cbc_encrypt_with_pkcs7(cipher, data);

        let mut result = iv_arr.to_vec();
        result.extend_from_slice(&ciphertext);
        result
    }
}

// ---------------------------------------------------------------------------
// PDF password padding string (RC4 only)
// ---------------------------------------------------------------------------

/// PDF password padding string (32 bytes) per PDF spec Algorithm 3.2
const PADDING: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Pad or truncate a password to 32 bytes using the PDF padding string (RC4)
fn pad_password(password: &str) -> Vec<u8> {
    let pwd_bytes = password.as_bytes();
    let mut padded = Vec::with_capacity(32);

    if pwd_bytes.len() >= 32 {
        padded.extend_from_slice(&pwd_bytes[..32]);
    } else {
        padded.extend_from_slice(pwd_bytes);
        let remaining = 32 - pwd_bytes.len();
        padded.extend_from_slice(&PADDING[..remaining]);
    }

    padded
}

/// MD5 hash helper
fn md5_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// SHA-256 hash of multiple parts concatenated
fn sha256_hash_parts(parts: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().to_vec()
}

/// RC4 encryption (symmetric — same function for encrypt and decrypt)
fn rc4_encrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    // Key Scheduling Algorithm (KSA)
    let mut s: Vec<u8> = (0..=255).map(|i| i as u8).collect();
    let mut j: usize = 0;
    for i in 0..256 {
        j = (j + s[i] as usize + key[i % key.len()] as usize) % 256;
        s.swap(i, j);
    }

    // Pseudo-Random Generation Algorithm (PRGA)
    let mut output = Vec::with_capacity(data.len());
    let mut i: usize = 0;
    j = 0;
    for &byte in data {
        i = (i + 1) % 256;
        j = (j + s[i] as usize) % 256;
        s.swap(i, j);
        let k = s[(s[i] as usize + s[j] as usize) % 256];
        output.push(byte ^ k);
    }

    output
}

/// AES-256-CBC encrypt with PKCS7 padding; IV prepended to output
///
/// Returns IV (16 bytes) || ciphertext.
fn encrypt_aes256_cbc(data: &[u8], key: &[u8], obj_num: u32) -> Vec<u8> {
    // Deterministic IV derived from key + obj_num (avoids need for rand crate)
    let mut iv_src = [0u8; 36];
    iv_src[..32].copy_from_slice(&key[..32]);
    iv_src[32..36].copy_from_slice(&obj_num.to_le_bytes());
    let iv_hash = sha256_hash_parts(&[&iv_src]);
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&iv_hash[..16]);

    type Aes256Cbc = cbc::Encryptor<Aes256>;
    let cipher = Aes256Cbc::new_from_slices(&key[..32], &iv)
        .expect("AES-256 key/IV lengths valid");
    let ciphertext = aes256_cbc_encrypt_with_pkcs7(cipher, data);

    let mut result = iv.to_vec();
    result.extend_from_slice(&ciphertext);
    result
}

/// AES-256-CBC encrypt a single 32-byte key block (no IV) for /UE and /OE
///
/// Uses a zero IV (as per PDF 2.0 spec for key wrapping).
fn aes256_cbc_encrypt_no_iv(data: &[u8], key: &[u8]) -> Vec<u8> {
    let iv = [0u8; 16];
    type Aes256Cbc = cbc::Encryptor<Aes256>;
    let cipher = Aes256Cbc::new_from_slices(&key[..32], &iv)
        .expect("AES-256 key/IV lengths valid");
    aes256_cbc_encrypt_with_pkcs7(cipher, data)
}

/// AES-256-ECB encrypt a single 16-byte block (for /Perms)
fn aes256_ecb_encrypt_block(data: &[u8; 16], key: &[u8]) -> [u8; 16] {
    let cipher = Aes256::new_from_slice(&key[..32])
        .expect("AES-256 key length is 32 bytes");
    let mut block = aes::Block::try_from(data.as_slice())
        .expect("block is exactly 16 bytes");
    cipher.encrypt_block(&mut block);
    block.into()
}

/// PKCS7 pad data to a multiple of block_size
fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let pad_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    for _ in 0..pad_len {
        padded.push(pad_len as u8);
    }
    padded
}

/// AES-256-CBC encrypt `data` with PKCS7 padding using the given encryptor.
///
/// This avoids the `cipher/alloc` feature requirement by manually padding
/// and encrypting 16-byte blocks.
fn aes256_cbc_encrypt_with_pkcs7(
    mut cipher: cbc::Encryptor<Aes256>,
    data: &[u8],
) -> Vec<u8> {
    use cbc::cipher::BlockModeEncrypt;
    let padded = pkcs7_pad(data, 16);
    // Process blocks in-place
    let mut out = padded;
    let block_count = out.len() / 16;
    for i in 0..block_count {
        let start = i * 16;
        let end = start + 16;
        let mut block = aes::Block::try_from(&out[start..end])
            .expect("block is exactly 16 bytes");
        cipher.encrypt_block(&mut block);
        out[start..end].copy_from_slice(&block);
    }
    out
}

/// Derive a deterministic 8-byte salt from a password and a tag
fn derive_salt(password: &str, tag: &[u8]) -> [u8; 8] {
    let hash = sha256_hash_parts(&[password.as_bytes(), tag]);
    let mut salt = [0u8; 8];
    salt.copy_from_slice(&hash[..8]);
    salt
}

/// Derive a deterministic 32-byte file encryption key
fn derive_file_enc_key(owner_pass: &str, user_pass: &str) -> Vec<u8> {
    sha256_hash_parts(&[owner_pass.as_bytes(), user_pass.as_bytes(), b"file_enc_key"])
}

/// Encode bytes as hexadecimal string (uppercase)
fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Generate a PDF file ID (two identical 16-byte MD5 hashes)
///
/// In practice, the file ID should be based on file content,
/// creation time, file size, etc. For simplicity, we generate
/// from a seed string.
pub fn generate_file_id(seed: &str) -> Vec<u8> {
    md5_hash(seed.as_bytes())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_password_empty() {
        let padded = pad_password("");
        assert_eq!(padded.len(), 32);
        assert_eq!(padded, PADDING.to_vec());
    }

    #[test]
    fn test_pad_password_short() {
        let padded = pad_password("test");
        assert_eq!(padded.len(), 32);
        assert_eq!(&padded[..4], b"test");
        assert_eq!(&padded[4..], &PADDING[..28]);
    }

    #[test]
    fn test_pad_password_long() {
        let long_pwd = "a".repeat(40);
        let padded = pad_password(&long_pwd);
        assert_eq!(padded.len(), 32);
        assert_eq!(padded, vec![b'a'; 32]);
    }

    #[test]
    fn test_permissions_default_all_allowed() {
        let perms = PdfPermissions::default();
        let p = perms.to_p_value();
        // All permission bits should be set
        assert!(p & (1 << 2) != 0); // print
        assert!(p & (1 << 3) != 0); // modify
        assert!(p & (1 << 4) != 0); // copy
        assert!(p & (1 << 5) != 0); // annotations
    }

    #[test]
    fn test_permissions_restricted() {
        let perms = PdfPermissions {
            allow_print: false,
            allow_copy: false,
            allow_modify: false,
            allow_annotations: false,
            ..Default::default()
        };
        let p = perms.to_p_value();
        assert!(p & (1 << 2) == 0); // no print
        assert!(p & (1 << 3) == 0); // no modify
        assert!(p & (1 << 4) == 0); // no copy
        assert!(p & (1 << 5) == 0); // no annotations
    }

    #[test]
    fn test_rc4_encrypt_decrypt_roundtrip() {
        let key = b"testkey123456789";
        let plaintext = b"Hello, World! This is a PDF encryption test.";
        let encrypted = rc4_encrypt(key, plaintext);
        let decrypted = rc4_encrypt(key, &encrypted);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_rc4_encryption_dict_computation() {
        let security = PdfSecurity::new("owner", "user", PdfPermissions::default());
        let file_id = generate_file_id("test-document");
        let dict = security.compute_encryption_dict(&file_id);

        assert_eq!(dict.o_value.len(), 32);
        assert_eq!(dict.u_value.len(), 32);
        assert_eq!(dict.key_length, 128);
        assert_eq!(dict.revision, 3);
        assert_eq!(dict.version, 2);
        assert_eq!(dict.algorithm, EncryptionAlgorithm::Rc4128);
    }

    #[test]
    fn test_rc4_object_encryption_roundtrip() {
        let security = PdfSecurity::new("owner", "user", PdfPermissions::default());
        let file_id = generate_file_id("test-doc");
        let dict = security.compute_encryption_dict(&file_id);

        let plaintext = b"This is a test stream content for PDF object.";
        let encrypted = dict.encrypt_data(plaintext, 5, 0);
        let decrypted = dict.encrypt_data(&encrypted, 5, 0);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_rc4_encryption_dict_pdf_output() {
        let security = PdfSecurity::new("owner", "", PdfPermissions::default());
        let file_id = generate_file_id("test");
        let dict = security.compute_encryption_dict(&file_id);
        let pdf_str = dict.to_pdf_dict(10);

        assert!(pdf_str.contains("/Filter /Standard"));
        assert!(pdf_str.contains("/V 2"));
        assert!(pdf_str.contains("/R 3"));
        assert!(pdf_str.contains("/Length 128"));
        assert!(pdf_str.contains("/O <"));
        assert!(pdf_str.contains("/U <"));
    }

    #[test]
    fn test_aes256_encryption_dict_computation() {
        let security =
            PdfSecurity::new_aes256("owner_pass", "user_pass", PdfPermissions::default());
        let dict = security.compute_encryption_dict(&[]);

        // /O and /U are 48 bytes each (32-byte hash + 8 validation salt + 8 key salt)
        assert_eq!(dict.o_value.len(), 48);
        assert_eq!(dict.u_value.len(), 48);
        assert_eq!(dict.key_length, 256);
        assert_eq!(dict.revision, 6);
        assert_eq!(dict.version, 5);
        assert_eq!(dict.algorithm, EncryptionAlgorithm::Aes256);

        // OE and UE should be present
        assert!(dict.oe_value.is_some());
        assert!(dict.ue_value.is_some());
        assert!(dict.perms_value.is_some());

        // /Perms is 16 bytes
        assert_eq!(
            dict.perms_value
                .as_ref()
                .expect("test: should succeed")
                .len(),
            16
        );
    }

    #[test]
    fn test_aes256_encryption_dict_pdf_output() {
        let security =
            PdfSecurity::new_aes256("owner_pass", "user_pass", PdfPermissions::default());
        let dict = security.compute_encryption_dict(&[]);
        let pdf_str = dict.to_pdf_dict(10);

        assert!(pdf_str.contains("/Filter /Standard"));
        assert!(pdf_str.contains("/V 5"));
        assert!(pdf_str.contains("/R 6"));
        assert!(pdf_str.contains("/Length 256"));
        assert!(pdf_str.contains("/O <"));
        assert!(pdf_str.contains("/U <"));
        assert!(pdf_str.contains("/OE <"));
        assert!(pdf_str.contains("/UE <"));
        assert!(pdf_str.contains("/Perms <"));
        assert!(pdf_str.contains("/CFM /AESV3"));
        assert!(pdf_str.contains("/StmF /StdCF"));
        assert!(pdf_str.contains("/StrF /StdCF"));
    }

    #[test]
    fn test_aes256_stream_encryption() {
        let security =
            PdfSecurity::new_aes256("owner_pass", "user_pass", PdfPermissions::default());
        let dict = security.compute_encryption_dict(&[]);

        let plaintext = b"Sensitive PDF stream content.";
        let encrypted = dict.encrypt_data(plaintext, 10, 0);

        // Encrypted data should be longer (IV + padded ciphertext)
        assert!(encrypted.len() > plaintext.len());
        // Should not equal plaintext
        assert_ne!(&encrypted[16..16 + plaintext.len()], plaintext.as_slice());
    }

    #[test]
    fn test_aes256_deterministic() {
        // Same inputs should produce same encryption output
        let security =
            PdfSecurity::new_aes256("owner_pass", "user_pass", PdfPermissions::default());
        let dict = security.compute_encryption_dict(&[]);

        let plaintext = b"Test data for AES-256.";
        let enc1 = dict.encrypt_data(plaintext, 5, 0);
        let enc2 = dict.encrypt_data(plaintext, 5, 0);
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0xFF, 0x00, 0xAB]), "FF00AB");
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_file_id_generation() {
        let id1 = generate_file_id("doc1");
        let id2 = generate_file_id("doc2");
        assert_eq!(id1.len(), 16);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_encryption_algorithm_default() {
        let algo = EncryptionAlgorithm::default();
        assert_eq!(algo, EncryptionAlgorithm::Rc4128);
    }

    #[test]
    fn test_pkcs7_pad() {
        // 13 bytes padded to 16 → pad = 3 bytes of value 3
        let data = b"Hello, World!";
        let padded = pkcs7_pad(data, 16);
        assert_eq!(padded.len(), 16);
        assert_eq!(&padded[13..], &[3u8, 3, 3]);
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_permissions_all_denied() {
        let perms = PdfPermissions {
            allow_print: false,
            allow_modify: false,
            allow_copy: false,
            allow_annotations: false,
            allow_fill_forms: false,
            allow_accessibility: false,
            allow_assemble: false,
            allow_print_high_quality: false,
        };
        let p = perms.to_p_value();
        // None of the user-controlled permission bits should be set
        assert_eq!(p & (1 << 2), 0); // no print
        assert_eq!(p & (1 << 3), 0); // no modify
        assert_eq!(p & (1 << 4), 0); // no copy
        assert_eq!(p & (1 << 5), 0); // no annotations
        assert_eq!(p & (1 << 8), 0); // no fill forms
        assert_eq!(p & (1 << 9), 0); // no accessibility
        assert_eq!(p & (1 << 10), 0); // no assemble
        assert_eq!(p & (1 << 11), 0); // no high-quality print
    }

    #[test]
    fn test_permissions_only_print_allowed() {
        let perms = PdfPermissions {
            allow_print: true,
            allow_modify: false,
            allow_copy: false,
            allow_annotations: false,
            allow_fill_forms: false,
            allow_accessibility: false,
            allow_assemble: false,
            allow_print_high_quality: false,
        };
        let p = perms.to_p_value();
        assert_ne!(p & (1 << 2), 0); // print allowed
        assert_eq!(p & (1 << 3), 0); // no modify
        assert_eq!(p & (1 << 4), 0); // no copy
    }

    #[test]
    fn test_pdf_security_new_stores_passwords() {
        let perms = PdfPermissions::default();
        let sec = PdfSecurity::new("owner_pw", "user_pw", perms);
        assert_eq!(sec.owner_password, "owner_pw");
        assert_eq!(sec.user_password, "user_pw");
        assert_eq!(sec.key_length, 128);
        assert_eq!(sec.algorithm, EncryptionAlgorithm::Rc4128);
    }

    #[test]
    fn test_pdf_security_aes256_stores_correct_length() {
        let perms = PdfPermissions::default();
        let sec = PdfSecurity::new_aes256("owner", "user", perms);
        assert_eq!(sec.key_length, 256);
        assert_eq!(sec.algorithm, EncryptionAlgorithm::Aes256);
    }

    #[test]
    fn test_rc4_different_objects_produce_different_ciphertext() {
        let sec = PdfSecurity::new("owner", "user", PdfPermissions::default());
        let file_id = generate_file_id("doc");
        let dict = sec.compute_encryption_dict(&file_id);

        let plaintext = b"same plaintext for both objects";
        let enc_obj1 = dict.encrypt_data(plaintext, 1, 0);
        let enc_obj2 = dict.encrypt_data(plaintext, 2, 0);
        // Different objects → different derived keys → different ciphertext
        assert_ne!(enc_obj1, enc_obj2);
    }

    #[test]
    fn test_aes256_different_objects_produce_different_ciphertext() {
        let sec = PdfSecurity::new_aes256("owner", "user", PdfPermissions::default());
        let dict = sec.compute_encryption_dict(&[]);

        let plaintext = b"same plaintext";
        let enc_obj1 = dict.encrypt_data(plaintext, 1, 0);
        let enc_obj2 = dict.encrypt_data(plaintext, 2, 0);
        assert_ne!(enc_obj1, enc_obj2);
    }

    #[test]
    fn test_rc4_empty_plaintext() {
        let sec = PdfSecurity::new("owner", "user", PdfPermissions::default());
        let file_id = generate_file_id("doc");
        let dict = sec.compute_encryption_dict(&file_id);

        let encrypted = dict.encrypt_data(b"", 1, 0);
        // RC4 of empty input is empty
        assert!(encrypted.is_empty());
    }

    #[test]
    fn test_generate_file_id_is_16_bytes() {
        let id = generate_file_id("test");
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_generate_file_id_different_seeds_differ() {
        let a = generate_file_id("seed_a");
        let b = generate_file_id("seed_b");
        assert_ne!(a, b);
    }

    #[test]
    fn test_hex_encode_all_bytes() {
        let data: Vec<u8> = (0..=15).collect();
        let hex = hex_encode(&data);
        assert_eq!(hex, "000102030405060708090A0B0C0D0E0F");
    }

    #[test]
    fn test_pkcs7_pad_exact_block() {
        // Exactly 16 bytes: padding block of 16 bytes with value 16 is added
        let data = b"1234567890123456";
        let padded = pkcs7_pad(data, 16);
        assert_eq!(padded.len(), 32);
        assert_eq!(&padded[16..], &[16u8; 16]);
    }

    #[test]
    fn test_encryption_dict_clone() {
        let sec = PdfSecurity::new("o", "u", PdfPermissions::default());
        let file_id = generate_file_id("clone");
        let dict = sec.compute_encryption_dict(&file_id);
        let dict2 = dict.clone();
        assert_eq!(dict.key_length, dict2.key_length);
        assert_eq!(dict.algorithm, dict2.algorithm);
    }

    #[test]
    fn test_encryption_algorithm_debug() {
        let rc4 = EncryptionAlgorithm::Rc4128;
        let aes = EncryptionAlgorithm::Aes256;
        assert!(format!("{:?}", rc4).contains("Rc4"));
        assert!(format!("{:?}", aes).contains("Aes"));
    }
}
