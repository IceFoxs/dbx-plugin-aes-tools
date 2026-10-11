use aes::Aes128;
use ecb::{Decryptor, Encryptor};
use cipher::{
    block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit,
};
use base64::{engine::general_purpose, Engine as _};

type Aes128EcbEnc = Encryptor<Aes128>;
type Aes128EcbDec = Decryptor<Aes128>;

/// AES-128-ECB 加密（PKCS7 填充），兼容 Hutool 默认配置
/// key: 16 个字符的字符串
/// 返回: Base64 字符串
pub fn aes_encrypt(plaintext: &str, key: &str) -> Result<String, String> {
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 16 {
        return Err(format!("The Key must be 16 bytes, currently {} bytes", key_bytes.len()));
    }

    let cipher = Aes128EcbEnc::new(key_bytes.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

    Ok(general_purpose::STANDARD.encode(&ciphertext))
}

/// AES-128-ECB 解密（PKCS7 填充），兼容 Hutool 默认配置
/// encrypted_base64: Base64 字符串
/// key: 16 个字符的字符串
pub fn aes_decrypt(encrypted_base64: &str, key: &str) -> Result<String, String> {
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 16 {
        return Err(format!("The Key must be 16 bytes, currently {} bytes", key_bytes.len()));
    }

    let ciphertext = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|e| format!("Base64 decoding failed: {}", e))?;

    let cipher = Aes128EcbDec::new(key_bytes.into());
    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 conversion failed: {}", e))
}