use crate::primitive::*;
use anyhow::{anyhow, Result};
use blake2::Digest;
use ed25519_dalek::Signer;

pub trait Cryptography {
    fn signer_public_key(&self) -> PublicKey;

    fn signature(&self) -> Signature;
    fn update_signature(&mut self, signature: Signature);

    fn as_data_for_signing(&self) -> Result<Vec<u8>>;

    /// Hash calculation
    fn hash(&self) -> Result<Hash> {
        let mut hasher = Blake2b256::new();

        let bytes = self.as_data_for_signing()?;
        hasher.update(bytes.as_slice());

        Ok(hasher.finalize().into())
    }

    /// Signing a data with a private key
    fn sign(&mut self, secret_key: &SecretKey) -> Result<()> {
        let secret_key = ed25519_dalek::SecretKey::from_bytes(secret_key.as_slice())
            .map_err(|error| anyhow!("Secret key serialization failed: {error:?}"))?;

        let public_key = ed25519_dalek::PublicKey::from(&secret_key);
        if public_key.to_bytes() != self.signer_public_key() {
            return Err(anyhow!(
                "The public key derived from the private key does not match to the public key in the data"
            ));
        }

        let keypair = ed25519_dalek::Keypair {
            secret: secret_key,
            public: public_key,
        };

        let message = self.hash()?;

        let signature = keypair.try_sign(message.as_slice())?;
        self.update_signature(signature.to_bytes());

        Ok(())
    }

    /// Signature verification
    fn signature_verify(&self) -> Result<()> {
        let public_key = ed25519_dalek::PublicKey::from_bytes(&self.signer_public_key())
            .map_err(|error| anyhow!("Public key serialization failed: {error:?}"))?;

        let message = self.hash()?;

        let signature = ed25519_dalek::Signature::from_bytes(&self.signature())
            .map_err(|error| anyhow!("Signature serialization failed: {error:?}"))?;

        public_key
            .verify_strict(message.as_slice(), &signature)
            .map_err(|error| anyhow!("Signature not valid: {error:?}"))
    }
}
