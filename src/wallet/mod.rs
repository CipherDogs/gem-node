use crate::{account::Account, primitive::*};
use anyhow::{anyhow, Result};
use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use base58::{FromBase58, ToBase58};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit};
use rand::rngs::OsRng;
use std::{
    fs::File,
    io::{Read, Write},
};

// Keypair and account generation
pub fn generate() -> (SecretKey, PublicKey) {
    let keypair = generate_keypair();

    let secret_key = keypair.secret.to_bytes();
    let public_key = keypair.public.to_bytes();

    let account = Account::from_public_key(public_key);

    println!("Secret key: {}", secret_key.to_base58());
    println!("Public key: {}", public_key.to_base58());
    println!("Address: {}", account.address.to_base58());

    (secret_key, public_key)
}

/// Importing the secret key from the base58 string
pub fn import(base58: &str) -> Result<(SecretKey, PublicKey)> {
    let bytes = base58
        .from_base58()
        .map_err(|error| anyhow!("Base58 decode failed: {error:?}"))?;
    let secret_key = ed25519_dalek::SecretKey::from_bytes(bytes.as_slice())
        .map_err(|error| anyhow!("Secret key serialization failed: {error:?}"))?;
    let public_key = ed25519_dalek::PublicKey::from(&secret_key);

    Ok((secret_key.to_bytes(), public_key.to_bytes()))
}

/// Loading a secret key from wallet.dat
pub fn load(wallet_path: &str) -> Result<(SecretKey, PublicKey)> {
    let password = rpassword::prompt_password("Wallet password: ")?;

    let mut file = File::open(wallet_path)?;

    let bytes = {
        let mut bytes = [0; 184];
        file.read_exact(&mut bytes)?;
        hex::decode(bytes)?
    };

    let mut salt = [0u8; 32];
    salt.copy_from_slice(&bytes[0..32]);
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&bytes[32..44]);
    let mut ciphertext = [0u8; 48];
    ciphertext.copy_from_slice(&bytes[44..92]);

    let cipher_key = argon2_key_derivation(password.as_bytes(), &salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&cipher_key)?;

    let plaintext = cipher
        .decrypt(&nonce.try_into()?, ciphertext.as_slice())
        .map_err(|error| anyhow!("ChaCha20Poly1305 decrypt failed: {error:?}"))?;

    let secret_key = ed25519_dalek::SecretKey::from_bytes(plaintext.as_slice())
        .map_err(|error| anyhow!("Secret key serialization failed: {error:?}"))?;

    let public_key = ed25519_dalek::PublicKey::from(&secret_key);

    Ok((secret_key.to_bytes(), public_key.to_bytes()))
}

/// Saving a secret key in wallet.dat
pub fn save(wallet_path: &str, secret_key: SecretKey) -> Result<()> {
    let password = rpassword::prompt_password("Wallet password: ")?;

    let salt: [u8; 32] = rand::random();
    let nonce: [u8; 12] = rand::random();

    let cipher_key = argon2_key_derivation(password.as_bytes(), &salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&cipher_key)?;

    let ciphertext = cipher
        .encrypt(&nonce.try_into()?, secret_key.as_slice())
        .map_err(|error| anyhow!("ChaCha20Poly1305 encrypt failed: {error:?}"))?;

    let mut bytes = [0; 92];
    bytes[0..32].copy_from_slice(&salt);
    bytes[32..44].copy_from_slice(&nonce);
    bytes[44..92].copy_from_slice(ciphertext.as_slice());

    let mut file = File::create(wallet_path)?;
    file.write_all(hex::encode(bytes).as_bytes())?;

    Ok(())
}

fn argon2_key_derivation(password: &[u8], salt: &[u8; 32]) -> Result<Hash> {
    let mut builder = ParamsBuilder::new();

    builder.m_cost(1024);
    builder.t_cost(1);
    builder.p_cost(1);

    let params = builder
        .build()
        .map_err(|error| anyhow!("Argon2 params build failed: {error:?}"))?;
    let ctx = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut bytes = [0; 32];
    ctx.hash_password_into(password, salt, &mut bytes)
        .map_err(|error| anyhow!("Argon2 hashing failed: {error:?}"))?;
    Ok(bytes)
}

/// Generation of a random ed25519 keypair
fn generate_keypair() -> ed25519_dalek::Keypair {
    let mut csprng = OsRng {};
    ed25519_dalek::Keypair::generate(&mut csprng)
}
