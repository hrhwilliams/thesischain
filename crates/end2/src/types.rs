use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Uuid as SqlUuid;
use uuid::Uuid;

use crate::AppError;

macro_rules! prefixed_uuid {
    ($name:ident, $prefix:expr) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            diesel::AsExpression,
            diesel::deserialize::FromSqlRow,
        )]
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new_v7() -> Self {
                Self(Uuid::now_v7())
            }

            #[must_use]
            pub const fn into_inner(self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = AppError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let prefix_with_sep = concat!($prefix, "_");
                if let Some(rest) = value.strip_prefix(prefix_with_sep) {
                    return Uuid::parse_str(rest)
                        .map(Self)
                        .map_err(|e| AppError::ValueError(e.to_string()));
                }

                Err(AppError::ValueError(format!(
                    "expected prefix '{}', got '{}'",
                    $prefix, value
                )))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}_{}", $prefix, self.0)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
            }
        }

        impl FromSql<SqlUuid, Pg> for $name {
            fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
                let uuid = <Uuid as FromSql<SqlUuid, Pg>>::from_sql(bytes)?;
                Ok(Self(uuid))
            }
        }

        impl ToSql<SqlUuid, Pg> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
                <Uuid as ToSql<SqlUuid, Pg>>::to_sql(&self.0, out)
            }
        }
    };
}

prefixed_uuid!(DeviceId, "dev");
prefixed_uuid!(UserId, "usr");
prefixed_uuid!(ChannelId, "ch");
prefixed_uuid!(MessageId, "msg");
prefixed_uuid!(SessionId, "sess");
prefixed_uuid!(OtkId, "otk");
prefixed_uuid!(DiscordInfoId, "di");
prefixed_uuid!(DiscordAuthTokenId, "dat");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_string_to_uuid() {
        assert_eq!(
            UserId::try_from("usr_00000000-0000-0000-0000-000000000000")
                .expect("parse")
                .into_inner(),
            Uuid::nil()
        );
        assert_eq!(
            DeviceId::try_from("dev_00000000-0000-0000-0000-000000000000")
                .expect("parse")
                .into_inner(),
            Uuid::nil()
        );
    }

    #[test]
    fn convert_to_raw_uuid() {
        assert_eq!(
            UserId::try_from("usr_00000000-0000-0000-0000-000000000000")
                .expect("parse")
                .into_inner(),
            Uuid::nil()
        );
    }

    #[test]
    fn display_format() {
        let id = UserId::from(Uuid::nil());
        assert_eq!(id.to_string(), "usr_00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn wrong_prefix_fails() {
        assert!(UserId::try_from("dev_00000000-0000-0000-0000-000000000000").is_err());
    }

    #[test]
    fn roundtrip_serde() {
        let id = ChannelId::from(Uuid::nil());
        let json = serde_json::to_string(&id).expect("serialize");
        assert_eq!(json, "\"ch_00000000-0000-0000-0000-000000000000\"");
        let parsed: ChannelId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, id);
    }
}
