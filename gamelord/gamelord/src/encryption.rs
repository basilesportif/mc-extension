use crypto::aead::{AeadDecryptor, AeadEncryptor};
use crypto::aes::KeySize::KeySize256;
use crypto::aes_gcm::AesGcm;
use crypto::hkdf::{hkdf_expand, hkdf_extract};
use crypto::sha2::Sha256;
use rand::{thread_rng, RngCore};

const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// these 2 are template encryption functions using rust-crypto-wasm to run
/// aes_gcm has trouble compiling to wasm using apple clang, it'd be cleaner with it
/// feel free to refactor better!

pub fn encrypt_data(data: &[u8], password: &str) -> Vec<u8> {
    let mut rng = thread_rng();

    // Generate salt
    let mut salt = [0u8; SALT_SIZE];
    rng.fill_bytes(&mut salt);

    // HKDF Extract
    let mut prk = [0u8; KEY_SIZE];
    hkdf_extract(Sha256::new(), &salt, password.as_bytes(), &mut prk);

    // HKDF Expand
    let mut okm = vec![0u8; KEY_SIZE];
    hkdf_expand(Sha256::new(), &prk, b"", &mut okm);

    // AES-GCM Encryption
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce);
    let mut encrypted_data = vec![0u8; data.len()];
    let mut tag = vec![0u8; 16]; // Tag size for AES-GCM
    let mut cipher = AesGcm::new(KeySize256, &okm, &nonce, &[]);
    cipher.encrypt(data, &mut encrypted_data, &mut tag);

    // Prefix salt and nonce to the encrypted data, append tag
    [
        salt.as_ref(),
        nonce.as_ref(),
        encrypted_data.as_ref(),
        tag.as_ref(),
    ]
    .concat()
}

pub fn decrypt_data(
    encrypted_data_with_nonce_salt_tag: &[u8],
    password: &str,
) -> Result<Vec<u8>, String> {
    if encrypted_data_with_nonce_salt_tag.len() < SALT_SIZE + NONCE_SIZE + 16 {
        // Ensure there's enough data for salt, nonce, and tag
        return Err("Encrypted data is too short".into());
    }

    // Extract salt, nonce, and tag from the input
    let salt = &encrypted_data_with_nonce_salt_tag[..SALT_SIZE];
    let nonce = &encrypted_data_with_nonce_salt_tag[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let tag = &encrypted_data_with_nonce_salt_tag[encrypted_data_with_nonce_salt_tag.len() - 16..];
    let encrypted_data = &encrypted_data_with_nonce_salt_tag
        [SALT_SIZE + NONCE_SIZE..encrypted_data_with_nonce_salt_tag.len() - 16];

    // HKDF Extract and Expand
    let mut prk = [0u8; KEY_SIZE];
    hkdf_extract(Sha256::new(), salt, password.as_bytes(), &mut prk);
    let mut okm = vec![0u8; KEY_SIZE];
    // Use a non-empty INFO if your application requires context-specific keys
    hkdf_expand(Sha256::new(), &prk, b"", &mut okm);

    // AES-GCM Decryption
    let mut decrypted_data = vec![0u8; encrypted_data.len()];
    let mut cipher = AesGcm::new(KeySize256, &okm, nonce, &[]);
    if cipher.decrypt(encrypted_data, &mut decrypted_data, tag) {
        Ok(decrypted_data)
    } else {
        Err("Decryption failed".into())
    }
}
