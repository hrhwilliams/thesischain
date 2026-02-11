use std::num::ParseIntError;

use argon2::password_hash::Encoding;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::ConnectionManager,
};
use r2d2::Pool;
use uuid::Uuid;

use crate::schema::{channel_participant, discord_info, user};
use crate::{
    AppError, InboundDiscordInfo, InboundUser, LoginError, NewDiscordInfo, NewUser,
    RegistrationError, User, is_valid_nickname,
};

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError>;
    async fn login(&self, username: &str, password: &str) -> Result<User, LoginError>;
    async fn login_with_discord(&self, info: &InboundDiscordInfo) -> Result<User, LoginError>;
    async fn register_with_discord(
        &self,
        info: InboundDiscordInfo,
    ) -> Result<User, RegistrationError>;
    async fn link_account(
        &self,
        user: &User,
        info: InboundDiscordInfo,
    ) -> Result<(), RegistrationError>;
    async fn get_user_info(&self, user_id: Uuid) -> Result<Option<User>, AppError>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError>;
    async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError>;
    async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError>;
}

pub struct DbAuthService {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbAuthService {
    #[must_use]
    pub const fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))
    }
}

#[async_trait]
impl AuthService for DbAuthService {
    async fn get_user_info(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let mut conn = self.get_conn()?;

        let user = tokio::task::spawn_blocking(move || {
            user::table
                .find(user_id)
                .select(User::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let mut conn = self.get_conn()?;

        let username = username.to_string();
        let user = tokio::task::spawn_blocking(move || {
            user::table
                .filter(user::username.eq(username))
                .select(User::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError> {
        let mut conn = self.get_conn()?;

        let user = tokio::task::spawn_blocking(move || {
            discord_info::table
                .inner_join(user::table)
                .filter(discord_info::discord_id.eq(discord_id))
                .select(User::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(user)
    }

    #[tracing::instrument(skip(self, password))]
    async fn login(&self, username: &str, password: &str) -> Result<User, LoginError> {
        tracing::info!("logging in user");
        let user = self
            .get_user_by_username(username)
            .await
            .map_err(LoginError::InternalError)?
            .ok_or(LoginError::NoSuchUser)?;

        let user_password = user.password.as_ref().ok_or(LoginError::NoPassword)?;

        let password_hash = PasswordHash::parse(user_password, Encoding::B64)
            .map_err(|e| AppError::ArgonError(e.to_string()))?;

        let is_correct = Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok();

        if is_correct {
            Ok(user)
        } else {
            Err(LoginError::InvalidPassword)
        }
    }

    #[tracing::instrument(skip(self))]
    async fn login_with_discord(
        &self,
        inbound_discord_info: &InboundDiscordInfo,
    ) -> Result<User, LoginError> {
        tracing::info!("logging in user via discord");
        let discord_id = inbound_discord_info
            .id
            .parse()
            .map_err(|e: ParseIntError| LoginError::InvalidDiscordId(e.to_string()))?;
        self.get_user_by_discord_id(discord_id)
            .await
            .map_err(LoginError::InternalError)?
            .ok_or(LoginError::NoSuchUser)
    }

    #[tracing::instrument(skip(self, inbound))]
    async fn register_with_discord(
        &self,
        inbound: InboundDiscordInfo,
    ) -> Result<User, RegistrationError> {
        let mut conn = self.get_conn().map_err(RegistrationError::InternalError)?;

        let new_user = NewUser {
            username: format!("{}@discord", inbound.username),
            password: None,
        };

        let user = diesel::insert_into(user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        let new_discord_info = NewDiscordInfo::from_inbound(inbound, user.id)?;

        diesel::insert_into(discord_info::table)
            .values(&new_discord_info)
            .execute(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        Ok(user)
    }

    #[tracing::instrument(skip(self, inbound))]
    async fn link_account(
        &self,
        user: &User,
        inbound: InboundDiscordInfo,
    ) -> Result<(), RegistrationError> {
        let mut conn = self.get_conn().map_err(RegistrationError::InternalError)?;

        let new_discord_info = NewDiscordInfo::from_inbound(inbound, user.id)?;

        diesel::insert_into(discord_info::table)
            .values(&new_discord_info)
            .execute(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, inbound))]
    async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError> {
        let new_user: NewUser = inbound.try_into()?;

        let mut conn = self.get_conn().map_err(RegistrationError::InternalError)?;

        let user = diesel::insert_into(user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .map_err(AppError::from)?;

        Ok(user)
    }

    async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError> {
        if !is_valid_nickname(nickname) {
            return Err(AppError::UserError("bad username".to_string()));
        }

        let mut conn = self.get_conn()?;

        let user_id = user.id;
        let nickname = nickname.to_string();
        tokio::task::spawn_blocking(move || {
            diesel::update(user::table.find(user_id))
                .set(user::nickname.eq(nickname.trim()))
                .execute(&mut conn)
        })
        .await??;

        Ok(())
    }

    async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError> {
        let mut conn = self.get_conn()?;

        let user_id = user.id;
        let users = tokio::task::spawn_blocking(move || {
            let channel_ids = channel_participant::table
                .filter(channel_participant::user_id.eq(user_id))
                .select(channel_participant::channel_id)
                .load::<Uuid>(&mut conn)?;

            channel_participant::table
                .inner_join(user::table)
                .filter(channel_participant::channel_id.eq_any(channel_ids))
                .distinct()
                .select(User::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(users)
    }
}
