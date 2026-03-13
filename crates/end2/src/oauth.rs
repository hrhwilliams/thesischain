use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
};

pub struct OAuthInfo {
    pub client_id: String,
    pub client_secret: String,
    pub redirect: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<&'static str>,
}

use crate::{InboundDiscordAuthToken, InboundDiscordInfo, OAuthError};

#[derive(Clone)]
pub struct OAuthHandler {
    client_id: String,
    client_secret: String,
    redirect: String,
    auth_url: String,
    token_url: String,
    scopes: Vec<&'static str>,
}

impl OAuthHandler {
    /// Creates a new `OAuthHandler`. The string arguments are intentionally leaked
    /// to obtain `'static` lifetimes, since this is constructed once at startup
    /// and lives for the entire process.
    #[must_use]
    pub fn new(info: OAuthInfo) -> Self {
        Self {
            client_id: info.client_id,
            client_secret: info.client_secret,
            redirect: info.redirect,
            auth_url: info.auth_url,
            token_url: info.token_url,
            scopes: info.scopes,
        }
    }

    pub fn generate_oauth_url(&self) -> Result<(String, CsrfToken, PkceCodeVerifier), OAuthError> {
        let client = BasicClient::new(ClientId::new(self.client_id.to_string()))
            .set_client_secret(ClientSecret::new(self.client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new(self.auth_url.clone())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_token_uri(
                TokenUrl::new(self.token_url.clone())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_redirect_uri(
                RedirectUrl::new(self.redirect.to_string())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            );

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut url_builder = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);
        for scope in &self.scopes {
            url_builder = url_builder.add_scope(Scope::new(scope.to_string()));
        }
        let (auth_url, csrf_token) = url_builder.url();

        Ok((auth_url.to_string(), csrf_token, pkce_verifier))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_google_token(
        &self,
        code: String,
        state: String,
        csrf_token: CsrfToken,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<InboundDiscordAuthToken, OAuthError> {
        if &state != csrf_token.secret() {
            return Err(OAuthError::StateMismatch);
        }

        let client = BasicClient::new(ClientId::new(self.client_id.to_string()))
            .set_client_secret(ClientSecret::new(self.client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new(self.auth_url.clone())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_token_uri(
                TokenUrl::new(self.token_url.clone())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_redirect_uri(
                RedirectUrl::new(self.redirect.to_string())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            );

        let http_client = oauth2::reqwest::ClientBuilder::new()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| OAuthError::FailedToBuildClient(e.to_string()))?;

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.clone()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&http_client)
            .await
            .map_err(|e| OAuthError::FailedToGetToken(e.to_string()))?;

        let discord_token = InboundDiscordAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
            expires: token_response.expires_in().map(|t| t.as_secs()),
        };

        Ok(discord_token)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_discord_token(
        &self,
        code: String,
        state: String,
        csrf_token: CsrfToken,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<InboundDiscordAuthToken, OAuthError> {
        if &state != csrf_token.secret() {
            return Err(OAuthError::StateMismatch);
        }

        let client = BasicClient::new(ClientId::new(self.client_id.to_string()))
            .set_client_secret(ClientSecret::new(self.client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new(self.auth_url.clone())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_token_uri(
                TokenUrl::new(self.token_url.clone())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_redirect_uri(
                RedirectUrl::new(self.redirect.to_string())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            );

        let http_client = oauth2::reqwest::ClientBuilder::new()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| OAuthError::FailedToBuildClient(e.to_string()))?;

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.clone()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&http_client)
            .await
            .map_err(|e| OAuthError::FailedToGetToken(e.to_string()))?;

        let discord_token = InboundDiscordAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
            expires: token_response.expires_in().map(|t| t.as_secs()),
        };

        Ok(discord_token)
    }

    pub async fn get_google_info(
        &self,
        google_token: &InboundDiscordAuthToken,
    ) -> Result<InboundDiscordInfo, OAuthError> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| OAuthError::FailedToBuildClient(e.to_string()))?;

        let user_info = http_client
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(google_token.access_token.clone())
            .send()
            .await
            .map_err(|_| OAuthError::FailedQuery)?
            .text()
            .await
            .map_err(|_| OAuthError::FailedQuery)?;

        tracing::info!(%user_info);
        Err(OAuthError::FailedQuery)
    }

    pub async fn get_discord_info(
        &self,
        discord_token: &InboundDiscordAuthToken,
    ) -> Result<InboundDiscordInfo, OAuthError> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| OAuthError::FailedToBuildClient(e.to_string()))?;

        let user_info: InboundDiscordInfo = http_client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(discord_token.access_token.clone())
            .send()
            .await
            .map_err(|_| OAuthError::FailedQuery)?
            .json()
            .await
            .map_err(|_| OAuthError::FailedQuery)?;

        Ok(user_info)
    }
}
