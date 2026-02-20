//! Cryptographic Provider Port
//!
//! Defines the interface for cryptographic operations used by providers
//! that need encryption capabilities (e.g., EncryptedVectorStoreProvider).
//!
//! ## Usage
//!
//! This port follows the Dependency Inversion Principle:
//! - The trait is defined here (mcb-domain)
//! - Implementations live in mcb-infrastructure (CryptoService)
//! - Providers depend on the abstraction, not the concrete implementation

use async_trait::async_trait;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Cryptographic provider port
///
/// Defines the contract for encryption/decryption operations.
/// Implementations provide the actual cryptographic primitives (e.g., AES-256-GCM).
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::{CryptoProvider, EncryptedData};
/// use std::sync::Arc;
///
/// fn encrypt_metadata(crypto: Arc<dyn CryptoProvider>, data: &[u8]) -> mcb_domain::Result<EncryptedData> {
///     crypto.encrypt(data)
/// }
/// ```
#[async_trait]
pub trait CryptoProvider: Send + Sync {
    /// Encrypt plaintext data
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The data to encrypt
    ///
    /// # Returns
    ///
    /// Encrypted data container with ciphertext and nonce
    ///
    /// # Errors
    /// Returns an error if encryption primitive fails.
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData>;

    /// Decrypt encrypted data
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - The encrypted data container
    ///
    /// # Returns
    ///
    /// The decrypted plaintext
    ///
    /// # Errors
    /// Returns an error if decryption primitive fails.
    fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>>;

    /// Get the name/identifier of this provider implementation
    fn provider_name(&self) -> &str;
}

/// Encrypted data container
///
/// Holds the ciphertext and nonce produced by encryption.
/// Can be serialized for storage in vector store metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
#[display(
    "EncryptedData {{ ciphertext: {} bytes, nonce: {} bytes }}",
    ciphertext.len(),
    nonce.len()
)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    /// Create a new encrypted data container
    #[must_use]
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self { ciphertext, nonce }
    }
}
