//! Protection du mot de passe et de la clé one-time (OpenSSL lié dans SFrame.exe) :
//! le client envoie une clé publique RSA-1024 (PEM « PUBLIC KEY »), le serveur répond
//! RSA-PKCS#1 v1.5(`aes_key[16] ‖ aes_iv[16]`), puis chaque message protégé est en
//! AES-128-CBC/PKCS#7, l'IV étant remis à sa valeur initiale à chaque message.

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use rsa::pkcs8::{DecodePublicKey, EncodePublicKey, LineEnding};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

type Enc = cbc::Encryptor<aes::Aes128>;
type Dec = cbc::Decryptor<aes::Aes128>;

#[derive(Debug, thiserror::Error)]
pub enum SecureError {
    #[error("clé publique RSA invalide : {0}")]
    BadPublicKey(String),
    #[error("RSA : {0}")]
    Rsa(#[from] rsa::Error),
    #[error("bloc AES invalide (padding)")]
    BadPadding,
    #[error("clé AES/IV de longueur {0} (32 attendus)")]
    BadKeyLength(usize),
}

pub type Result<T> = std::result::Result<T, SecureError>;

#[derive(Clone)]
pub struct AesSession {
    key: [u8; 16],
    iv: [u8; 16],
}

impl AesSession {
    pub fn random() -> Self {
        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut iv);
        Self { key, iv }
    }

    pub fn from_key_iv(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 32 {
            return Err(SecureError::BadKeyLength(bytes.len()));
        }
        Ok(Self {
            key: bytes[..16].try_into().unwrap(),
            iv: bytes[16..32].try_into().unwrap(),
        })
    }

    pub fn key_iv(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[..16].copy_from_slice(&self.key);
        out[16..].copy_from_slice(&self.iv);
        out
    }

    pub fn encrypt(&self, plain: &[u8]) -> Vec<u8> {
        Enc::new(&self.key.into(), &self.iv.into()).encrypt_padded_vec_mut::<Pkcs7>(plain)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Dec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(data)
            .map_err(|_| SecureError::BadPadding)
    }

    /// Côté serveur : chiffre clé+IV pour la clé publique PEM envoyée par le client.
    pub fn seal_for(&self, client_pem: &[u8]) -> Result<Vec<u8>> {
        let pem = String::from_utf8_lossy(client_pem);
        let pem = pem.trim_end_matches('\0');
        let public = RsaPublicKey::from_public_key_pem(pem).map_err(|e| SecureError::BadPublicKey(e.to_string()))?;
        Ok(public.encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, &self.key_iv())?)
    }
}

/// Côté client : paire RSA-1024 éphémère, comme SFrame.exe.
pub struct ClientKeyPair {
    private: RsaPrivateKey,
}

impl ClientKeyPair {
    pub fn generate() -> Result<Self> {
        Ok(Self { private: RsaPrivateKey::new(&mut rand::thread_rng(), 1024)? })
    }

    pub fn public_pem(&self) -> Vec<u8> {
        RsaPublicKey::from(&self.private)
            .to_public_key_pem(LineEnding::LF)
            .expect("export PEM")
            .into_bytes()
    }

    pub fn open(&self, sealed: &[u8]) -> Result<AesSession> {
        AesSession::from_key_iv(&self.private.decrypt(Pkcs1v15Encrypt, sealed)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_roundtrip() {
        let client = ClientKeyPair::generate().unwrap();
        let server_session = AesSession::random();
        let sealed = server_session.seal_for(&client.public_pem()).unwrap();
        assert_eq!(sealed.len(), 128);
        let client_session = client.open(&sealed).unwrap();
        let enc = client_session.encrypt(b"motdepasse");
        assert_eq!(enc.len(), 16);
        assert_eq!(server_session.decrypt(&enc).unwrap(), b"motdepasse");
        // L'IV est réinitialisé à chaque message : même entrée → même sortie.
        assert_eq!(client_session.encrypt(b"motdepasse"), enc);
    }
}
