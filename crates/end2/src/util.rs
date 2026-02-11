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

#[must_use]
pub fn is_valid_username(name: &str) -> bool {
    if name.len() < 3 {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

#[must_use]
pub fn is_valid_nickname(name: &str) -> bool {
    if name.len() < 3 {
        return false;
    }

    name.chars().all(|c| !c.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_usernames() {
        assert!(is_valid_username("abc"));
        assert!(is_valid_username("user_name"));
        assert!(is_valid_username("a.b.c"));
        assert!(is_valid_username("abc123"));
        assert!(is_valid_username("aaa"));
    }

    #[test]
    fn invalid_usernames() {
        assert!(!is_valid_username("ab"));
        assert!(!is_valid_username("a"));
        assert!(!is_valid_username(""));
        assert!(!is_valid_username("UPPERCASE"));
        assert!(!is_valid_username("has space"));
        assert!(!is_valid_username("a!b"));
    }

    #[test]
    fn valid_nicknames() {
        assert!(is_valid_nickname("Abc"));
        assert!(is_valid_nickname("A B C"));
        assert!(is_valid_nickname("nickname!"));
    }

    #[test]
    fn invalid_nicknames() {
        assert!(!is_valid_nickname("ab"));
        assert!(!is_valid_nickname("a"));
        assert!(!is_valid_nickname(""));
        assert!(!is_valid_nickname("ab\0c"));
    }
}
