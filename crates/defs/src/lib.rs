use bincode::Encode;
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
// use hmac::Hmac;
// use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::digest::typenum::U64;
use sha2::{Digest, Sha512, digest::generic_array::GenericArray};

#[derive(Encode, Debug, Deserialize, Serialize)]
pub struct Identity {
    username: String,
    #[bincode(with_serde)]
    verifying_key: VerifyingKey,
    #[bincode(with_serde)]
    hash: GenericArray<u8, U64>,
    #[bincode(with_serde)]
    signature: Signature,
}

impl Identity {
    pub fn new(username: String, signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();

        let mut hash: Sha512 = Sha512::new();
        hash.update(username.as_bytes());
        hash.update(verifying_key.as_bytes());

        let context = b"ThesisChain";
        let signature = signing_key
            .sign_prehashed(hash.clone(), Some(context))
            .expect("signature");

        Self {
            username,
            verifying_key,
            hash: hash.finalize().into(),
            signature,
        }
    }

    pub fn verify(&self) -> bool {
        let mut hasher = Sha512::new();
        hasher.update(self.username.as_bytes());
        hasher.update(self.verifying_key.as_bytes());

        if hasher.clone().finalize() != self.hash {
            return false;
        }

        let context = b"ThesisChain";
        self.verifying_key
            .verify_prehashed(hasher, Some(context), &self.signature)
            .is_ok()
    }
}
