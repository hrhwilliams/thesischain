use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
};

use crate::{InboundDiscordAuthToken, InboundDiscordInfo, OAuthError};

const DISCORD_AUTHORIZE_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";

type ConfiguredClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Clone)]
pub struct OAuthHandler {
    client_id: &'static str,
    client_secret: &'static str,
    redirect: &'static str,
}

impl OAuthHandler {
    /// Creates a new `OAuthHandler`. The string arguments are intentionally leaked
    /// to obtain `'static` lifetimes, since this is constructed once at startup
    /// and lives for the entire process.
    #[must_use]
    pub fn new(client_id: String, client_secret: String, redirect: String) -> Self {
        Self {
            client_id: client_id.leak(),
            client_secret: client_secret.leak(),
            redirect: redirect.leak(),
        }
    }

    fn build_client(&self) -> Result<ConfiguredClient, OAuthError> {
        Ok(BasicClient::new(ClientId::new(self.client_id.to_string()))
            .set_client_secret(ClientSecret::new(self.client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new(DISCORD_AUTHORIZE_URL.to_string())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_token_uri(
                TokenUrl::new(DISCORD_TOKEN_URL.to_string())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            )
            .set_redirect_uri(
                RedirectUrl::new(self.redirect.to_string())
                    .map_err(|_| OAuthError::FailedToCreateAuthUrl)?,
            ))
    }

    pub fn generate_oauth_url(&self) -> Result<(String, CsrfToken, PkceCodeVerifier), OAuthError> {
        let client = self.build_client()?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((auth_url.to_string(), csrf_token, pkce_verifier))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_token(
        &self,
        code: String,
        state: String,
        csrf_token: CsrfToken,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<InboundDiscordAuthToken, OAuthError> {
        if &state != csrf_token.secret() {
            return Err(OAuthError::StateMismatch);
        }

        let client = self.build_client()?;

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
