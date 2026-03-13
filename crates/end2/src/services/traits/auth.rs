use async_trait::async_trait;
use secrecy::SecretString;

use crate::{AppError, InboundDiscordInfo, InboundUser, LoginError, OAuthHandler, RegistrationError, User, UserId};

/// How the backend authenticates users and stores/distributes user info
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError>;
    async fn login(&self, username: &str, password: SecretString) -> Result<User, LoginError>;
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
    async fn get_user_info(&self, user_id: UserId) -> Result<Option<User>, AppError>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError>;
    async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError>;
    async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError>;
    fn get_oauth_handler(&self, service: &str) -> Option<OAuthHandler>;
}