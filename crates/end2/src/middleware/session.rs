use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, Expiration, SameSite},
};
use time::Duration;

use crate::{
    AppState, AuthService, DeviceKeyService, MessageRelayService, OtkService, SessionId,
    WebSessionService,
};

pub async fn create_session<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let (mut parts, body) = req.into_parts();
    let jar = CookieJar::from_request_parts(&mut parts, &app_state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let req = Request::from_parts(parts, body);

    if let Some(session_cookie) = jar.get("Session") {
        if let Ok(session_id) = SessionId::try_from(session_cookie.value()) {
            if app_state
                .get_session(session_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .is_some()
            {
                return Ok((jar, next.run(req).await));
            }
        }
    }

    let session = app_state
        .new_session()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("created new session {}", session.id);

    let cookie = Cookie::build(("Session", session.id.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .expires(Expiration::Session)
        .max_age(Duration::days(7))
        .build();
    Ok((jar.add(cookie), next.run(req).await))
}
