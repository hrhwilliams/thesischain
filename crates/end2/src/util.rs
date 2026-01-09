use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::Serializer;

pub fn serialize_as_base64<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD_NO_PAD.encode(bytes))
}
