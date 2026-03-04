//! Cryptography Tests
#![allow(clippy::expect_used)]

use mcb_infrastructure::crypto::{CryptoService, PasswordService, TokenGenerator};
use mcb_utils::utils::crypto::{HashUtils, SecureErasure};
use rstest::{fixture, rstest};

#[fixture]
fn master_key() -> Vec<u8> {
    CryptoService::generate_master_key()
}

#[fixture]
fn crypto_service(master_key: Vec<u8>) -> CryptoService {
    CryptoService::new(master_key).expect("CryptoService should initialize with valid key")
}

#[fixture]
fn password_service() -> PasswordService {
    PasswordService::new()
}

#[rstest]
fn test_crypto_service_encrypt_decrypt(crypto_service: CryptoService) {
    let plaintext = b"Hello, World!";
    let encrypted = crypto_service.encrypt(plaintext).unwrap();
    let decrypted = crypto_service.decrypt(&encrypted).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}

#[rstest]
fn test_crypto_service_invalid_key_size() {
    let invalid_key = vec![0u8; 16]; // Wrong size
    assert!(CryptoService::new(invalid_key).is_err());
}

#[rstest]
fn test_password_service_hash_verify(password_service: PasswordService) {
    let password = "test_password_123";
    let hash = password_service.hash_password(password).unwrap();

    assert!(password_service.verify_password(password, &hash).unwrap());
    assert!(
        !password_service
            .verify_password("wrong_password", &hash)
            .unwrap()
    );
}

#[rstest]
fn test_token_generator() {
    let token1 = TokenGenerator::generate_secure_token(32);
    let token2 = TokenGenerator::generate_secure_token(32);

    assert_eq!(token1.len(), 64); // 32 bytes * 2 hex chars
    assert_eq!(token2.len(), 64);
    assert_ne!(token1, token2);
}

#[rstest]
fn test_hash_utils_hmac() {
    let key = b"secret_key";
    let data = b"test_data";
    let hmac1 = HashUtils::hmac_sha256(key, data).expect("HMAC should succeed");
    let hmac2 = HashUtils::hmac_sha256(key, data).expect("HMAC should succeed");

    assert_eq!(hmac1, hmac2);
    assert_eq!(hmac1.len(), 32); // SHA256 output size
}

#[rstest]
fn test_secure_erasure() {
    let mut data = vec![1, 2, 3, 4, 5];
    SecureErasure::zeroize(&mut data);
    assert_eq!(data, vec![0, 0, 0, 0, 0]);
}

#[rstest]
#[case(b"test", b"test", true)]
#[case(b"test", b"different", false)]
#[case(b"test", b"test_longer", false)]
fn constant_time_eq(
    #[case] left: &'static [u8],
    #[case] right: &'static [u8],
    #[case] expected: bool,
) {
    assert_eq!(HashUtils::constant_time_eq(left, right), expected);
}
