use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::Serializer;

pub fn serialize_as_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD_NO_PAD.encode(bytes))
}

pub fn serialize_as_base64_opt<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match bytes {
        Some(b) => serializer.serialize_str(&BASE64_STANDARD_NO_PAD.encode(b)),
        None => serializer.serialize_none(),
    }
}

pub fn is_valid_username(name: &str) -> bool {
    if name.len() < 3 || name.len() > 20 {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

pub fn is_valid_nickname(name: &str) -> bool {
    if name.trim().len() < 3 || name.len() > 20 {
        return false;
    }

    name.chars().all(|c| !c.is_control())
}
