use crate::{CryptoError, HashAlgorithm};
use sha3::{Sha3_256, Sha3_512, Digest};
use std::io::Read;
use std::path::Path;

pub struct Hasher;

impl Hasher {
    /// Hash bytes with the specified algorithm
    pub fn hash_bytes(algorithm: &HashAlgorithm, data: &[u8]) -> String {
        match algorithm {
            HashAlgorithm::Sha3_256 => {
                let mut hasher = Sha3_256::new();
                hasher.update(data);
                hex::encode(hasher.finalize())
            }
            HashAlgorithm::Sha3_512 => {
                let mut hasher = Sha3_512::new();
                hasher.update(data);
                hex::encode(hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let hash = blake3::hash(data);
                hash.to_hex().to_string()
            }
        }
    }

    /// Hash a file with the specified algorithm
    pub fn hash_file(algorithm: &HashAlgorithm, path: &Path) -> Result<String, CryptoError> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| CryptoError::SerializationError(format!("Cannot open file: {}", e)))?;

        match algorithm {
            HashAlgorithm::Sha3_256 => {
                let mut hasher = Sha3_256::new();
                let mut buffer = [0u8; 8192];
                loop {
                    let bytes_read = file.read(&mut buffer)
                        .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
                    if bytes_read == 0 { break; }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(hex::encode(hasher.finalize()))
            }
            HashAlgorithm::Sha3_512 => {
                let mut hasher = Sha3_512::new();
                let mut buffer = [0u8; 8192];
                loop {
                    let bytes_read = file.read(&mut buffer)
                        .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
                    if bytes_read == 0 { break; }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(hex::encode(hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = blake3::Hasher::new();
                let mut buffer = [0u8; 8192];
                loop {
                    let bytes_read = file.read(&mut buffer)
                        .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
                    if bytes_read == 0 { break; }
                    hasher.update(&buffer[..bytes_read]);
                }
                Ok(hasher.finalize().to_hex().to_string())
            }
        }
    }

    /// Verify that a file matches an expected hash
    pub fn verify_file_hash(
        algorithm: &HashAlgorithm,
        path: &Path,
        expected_hash: &str,
    ) -> Result<bool, CryptoError> {
        let actual_hash = Self::hash_file(algorithm, path)?;
        if actual_hash == expected_hash {
            Ok(true)
        } else {
            Err(CryptoError::HashMismatch {
                expected: expected_hash.to_string(),
                actual: actual_hash,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let data = b"Hello, KryptoUpdate!";
        let hash1 = Hasher::hash_bytes(&HashAlgorithm::Sha3_256, data);
        let hash2 = Hasher::hash_bytes(&HashAlgorithm::Sha3_256, data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_data_different_hash() {
        let hash1 = Hasher::hash_bytes(&HashAlgorithm::Blake3, b"data1");
        let hash2 = Hasher::hash_bytes(&HashAlgorithm::Blake3, b"data2");
        assert_ne!(hash1, hash2);
    }
}