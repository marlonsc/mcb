//! Cryptographic provider ports.

use async_trait::async_trait;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Encrypted data container
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
#[display(
    "EncryptedData {{ ciphertext: {} bytes, nonce: {} bytes }}",
    ciphertext.len(),
    nonce.len()
)]
pub struct EncryptedData {
    /// The encrypted byte sequence.
    pub ciphertext: Vec<u8>,
    /// The initialization vector/nonce used for encryption.
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    /// Create new encrypted data from ciphertext and nonce.
    #[must_use]
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self { ciphertext, nonce }
    }
}

/// Cryptographic provider port
#[async_trait]
pub trait CryptoProvider: Send + Sync {
    /// Encrypt the given plaintext bytes.
    ///
    /// # Errors
    /// Returns an error if encryption fails.
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData>;

    /// Decrypt the given encrypted data.
    ///
    /// # Errors
    /// Returns an error if decryption fails or data is invalid.
    fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>>;

    /// Get the name of this cryptographic provider.
    fn provider_name(&self) -> &str;
}
