use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use crate::core::error::{CommonError, CoreError};

const FIXED_SALT: &[u8] = b"RdataStation_Connection_Vault_2026";

fn derive_key() -> [u8; 32] {
    let machine_id = get_machine_id();

    let mut hasher = Sha256::new();
    hasher.update(FIXED_SALT);
    hasher.update(machine_id.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn machine_id_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("RdataStation");
    path.push("machine-id");
    path
}

fn get_machine_id() -> String {
    let id_path = machine_id_path();
    if let Ok(id) = fs::read_to_string(&id_path) {
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    let fallback = build_fallback_id();
    if let Some(parent) = id_path.parent() {
        let _ = fs::create_dir_all(parent);
        let _ = fs::write(&id_path, &fallback);
    }
    fallback
}

fn build_fallback_id() -> String {
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string());

    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown-user".to_string());

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown-home".to_string());

    format!("{}:{}:{}", hostname, user, home)
}

pub fn encrypt_password(password: &str) -> Result<String, CoreError> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::common(CommonError::Internal(format!("AES init error: {}", e))))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, password.as_bytes()).map_err(|e| {
        CoreError::common(CommonError::Internal(format!("Encryption error: {}", e)))
    })?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &combined,
    ))
}

pub fn decrypt_password(encrypted: &str) -> Result<String, CoreError> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::common(CommonError::Internal(format!("AES init error: {}", e))))?;

    let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted)
        .map_err(|e| {
            CoreError::common(CommonError::Internal(format!("Base64 decode error: {}", e)))
        })?;

    if combined.len() < 12 {
        return Err(CoreError::common(CommonError::Internal(
            "Invalid encrypted data: too short".to_string(),
        )));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        CoreError::common(CommonError::Internal(format!("Decryption error: {}", e)))
    })?;

    String::from_utf8(plaintext)
        .map_err(|e| CoreError::common(CommonError::Internal(format!("UTF-8 decode error: {}", e))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "MySecretPassword123!";
        let encrypted = encrypt_password(original).expect("encrypt failed");
        let decrypted = decrypt_password(&encrypted).expect("decrypt failed");
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_empty_password() {
        let original = "";
        let encrypted = encrypt_password(original).expect("encrypt failed");
        let decrypted = decrypt_password(&encrypted).expect("decrypt failed");
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_unicode_password() {
        let original = "密码测试🔐";
        let encrypted = encrypt_password(original).expect("encrypt failed");
        let decrypted = decrypt_password(&encrypted).expect("decrypt failed");
        assert_eq!(original, decrypted);
    }
}
