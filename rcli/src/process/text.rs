use std::{fs, io::Read, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::{get_reader, process_genpass, TextFormat};

pub trait TextSign {
    /// Sign the data from the reader and return the signature.
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    /// Verify the data from the reader with the signature.
    fn verify(&self, reader: impl Read, signature: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    /// Load the key from the path.
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    /// Generate a new key.
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub trait Crypto {
    /// Encrypt the data from the reader and return the ciphertext.
    fn encrypt(&self, reader: &mut dyn Read) -> Result<String>;

    /// Decrypt the ciphertext from the reader and return the plaintext.
    fn decrypt(&self, reader: &mut dyn Read) -> Result<String>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub struct ChaCha20Poly1305Key {
    key: Key,
    nonce: XNonce,
}

pub fn process_text_sign(input: &str, key: &str, format: TextFormat) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;

    let signed = match format {
        TextFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
        _ => return Err(anyhow::anyhow!("Invalid text sign format")),
    };
    let signed = URL_SAFE_NO_PAD.encode(signed);
    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    signature: &str,
    format: TextFormat,
) -> anyhow::Result<bool> {
    let mut reader = get_reader(input)?;
    let signature = URL_SAFE_NO_PAD.decode(signature)?;
    let verified = match format {
        TextFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &signature)?
        }
        TextFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &signature)?
        }
        _ => return Err(anyhow::anyhow!("Invalid text sign format")),
    };
    Ok(verified)
}

pub fn process_text_generate(format: TextFormat) -> anyhow::Result<Vec<Vec<u8>>> {
    match format {
        TextFormat::Blake3 => Blake3::generate(),
        TextFormat::Ed25519 => Ed25519Signer::generate(),
        TextFormat::ChaCha20Poly1305 => ChaCha20Poly1305Key::generate(),
    }
}

pub fn process_text_encrypt(input: &str, key: &str) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let cipher = ChaCha20Poly1305Key::load(key)?;
    let ciphertext = cipher.encrypt(&mut reader)?;
    Ok(ciphertext)
}

pub fn process_text_decrypt(input: &str, key: &str) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let cipher = ChaCha20Poly1305Key::load(key)?;
    let plaintext = cipher.decrypt(&mut reader)?;
    Ok(plaintext)
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        Ok(hash.as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        Ok(hash.as_bytes() == signature)
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature = self.key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(signature.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();
        Ok(ret)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Blake3::try_new(&key)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for ChaCha20Poly1305Key {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        Ok(vec![key.as_bytes().to_vec()])
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();

        Ok(vec![sk, pk])
    }
}

impl KeyGenerator for ChaCha20Poly1305Key {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let key = XChaCha20Poly1305::generate_key(&mut csprng);
        let nonce = XChaCha20Poly1305::generate_nonce(&mut csprng);
        let mut key = key.to_vec();
        let nonce = nonce.to_vec();
        key.append(&mut nonce.to_vec());
        Ok(vec![key])
    }
}

impl Crypto for ChaCha20Poly1305Key {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<String> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let cipher = XChaCha20Poly1305::new(&self.key);
        let ciphertext = cipher.encrypt(&self.nonce, buf.as_ref()).unwrap();
        let ciphertext = URL_SAFE_NO_PAD.encode(ciphertext);
        Ok(ciphertext)
    }

    fn decrypt(&self, reader: &mut dyn Read) -> Result<String> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let text = URL_SAFE_NO_PAD.decode(&buf).unwrap_or(buf);
        let cipher = XChaCha20Poly1305::new(&self.key);
        let plaintext = cipher
            .decrypt(&self.nonce, text.as_ref())
            .expect("Invalid ciphertext");
        Ok(String::from_utf8(plaintext)?)
    }
}

impl Blake3 {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl Ed25519Signer {
    fn new(key: SigningKey) -> Self {
        Self { key }
    }

    fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        let signer = Ed25519Signer::new(key);
        Ok(signer)
    }
}

impl Ed25519Verifier {
    fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?);
        let verifier = Ed25519Verifier::new(key.unwrap());
        Ok(verifier)
    }
}

impl ChaCha20Poly1305Key {
    fn new(key: Key, nonce: XNonce) -> Self {
        Self { key, nonce }
    }

    fn try_new(data: &[u8]) -> Result<Self> {
        let key = &data[..32];
        let nonce = &data[32..56];
        let key = Key::clone_from_slice(key);
        let nonce = XNonce::clone_from_slice(nonce);
        let key = ChaCha20Poly1305Key::new(key, nonce);
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // test blake3 sign and verify
    fn test_blake3_sign_verify() -> Result<()> {
        let blake3 = Blake3::load("fixtures/blake3.txt")?;

        let data = b"hello world";
        let signature = blake3.sign(&mut &data[..])?;
        assert!(blake3.verify(&mut &data[..], &signature)?);
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let sk = Ed25519Signer::load("fixtures/ed25519.sk")?;
        let pk = Ed25519Verifier::load("fixtures/ed25519.pk")?;

        let data = b"hello world";
        let signature = sk.sign(&mut &data[..])?;
        assert!(pk.verify(&mut &data[..], &signature)?);
        Ok(())
    }

    #[test]
    fn test_chacha20poly1305_encrypt_decrypt() -> Result<()> {
        let key = ChaCha20Poly1305Key::load("fixtures/chacha20poly1305.txt")?;

        let data = b"hello world";
        let ciphertext = key.encrypt(&mut &data[..])?;
        let plaintext = key.decrypt(&mut ciphertext.as_bytes())?;
        assert_eq!(data, plaintext.as_bytes());
        Ok(())
    }
}
