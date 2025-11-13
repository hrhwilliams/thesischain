use crate::proto::IdentityTx;
use anyhow::{Context, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use ecdsa::RecoveryId;
use p256::{
    EncodedPoint, PublicKey, SecretKey,
    ecdsa::{Signature, SigningKey, VerifyingKey},
    elliptic_curve::{
        rand_core::OsRng,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    #[serde(
        serialize_with = "serialize_public_key_base64",
        deserialize_with = "deserialize_public_key_base64"
    )]
    pub public_key: PublicKey,
    #[serde(
        serialize_with = "serialize_signature_base64",
        deserialize_with = "deserialize_signature_base64"
    )]
    pub signature: Signature,
    pub id: u8,
}

fn serialize_public_key_base64<S>(public_key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let point = public_key.to_encoded_point(true);
    let encoded = BASE64_STANDARD.encode(point.as_bytes());
    serializer.serialize_str(&encoded)
}

fn deserialize_public_key_base64<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let bytes = BASE64_STANDARD
        .decode(encoded)
        .map_err(serde::de::Error::custom)?;
    let encoded_point = EncodedPoint::from_bytes(bytes).map_err(serde::de::Error::custom)?;
    PublicKey::from_encoded_point(&encoded_point)
        .into_option()
        .ok_or_else(|| serde::de::Error::custom("invalid public key"))
}

fn serialize_signature_base64<S>(signature: &Signature, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = BASE64_STANDARD.encode(signature.to_bytes());
    serializer.serialize_str(&encoded)
}

fn deserialize_signature_base64<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let bytes = BASE64_STANDARD
        .decode(encoded)
        .map_err(serde::de::Error::custom)?;
    Signature::try_from(bytes.as_slice()).map_err(|_| serde::de::Error::custom("invalid signature"))
}

impl Identity {
    pub fn new(name: String) -> Self {
        let private_key = SecretKey::random(&mut OsRng);
        let signing_key = SigningKey::from_bytes(&private_key.to_bytes()).unwrap();
        let public_key = private_key.public_key();
        let timestamp = OffsetDateTime::now_utc();

        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(timestamp.format(&Rfc3339).unwrap().as_bytes());
        hasher.update(public_key.to_encoded_point(true).as_bytes());

        let (signature, id) = signing_key.sign_digest_recoverable(hasher.clone()).unwrap();

        Self {
            name,
            timestamp,
            public_key,
            signature: signature,
            id: id.to_byte(),
        }
    }

    /// Non-interactive proof-of-possession algorithm that verifies the party
    /// publishing the identity possesses the private key for the given public key
    /// by checking that the verifying key of the signature matches the public key
    pub fn verify(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        hasher.update(self.timestamp.format(&Rfc3339).unwrap().as_bytes());
        hasher.update(self.public_key.to_encoded_point(true).as_bytes());

        let verifying_key = VerifyingKey::recover_from_digest(
            hasher,
            &self.signature,
            RecoveryId::from_byte(self.id).unwrap(),
        )
        .unwrap();

        verifying_key == self.public_key.into()
    }

    pub fn try_from_proto(tx: IdentityTx) -> anyhow::Result<Self> {
        let timestamp =
            OffsetDateTime::parse(&tx.timestamp, &Rfc3339).context("invalid timestamp")?;

        let public_key_bytes = BASE64_STANDARD
            .decode(tx.public_key)
            .context("public key base64 decode failed")?;
        let encoded_point = EncodedPoint::from_bytes(public_key_bytes)
            .map_err(|err| anyhow!("invalid public key encoding: {err}"))?;
        let public_key = PublicKey::from_encoded_point(&encoded_point)
            .into_option()
            .ok_or_else(|| anyhow!("public key is not on the curve"))?;

        let signature_bytes = BASE64_STANDARD
            .decode(tx.signature)
            .context("signature base64 decode failed")?;
        let signature = Signature::try_from(signature_bytes.as_slice())
            .map_err(|_| anyhow!("invalid signature bytes"))?;

        let id = if tx.recovery_id { 1 } else { 0 };

        Ok(Self {
            name: tx.identifier,
            timestamp,
            public_key,
            signature,
            id,
        })
    }
}
