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

use crate::{AppState, SessionId};

pub async fn create_session(
    State(app_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let (mut parts, body) = req.into_parts();
    let jar = CookieJar::from_request_parts(&mut parts, &app_state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut req = Request::from_parts(parts, body);

    if let Some(session_cookie) = jar.get("Session") {
        if let Ok(session_id) = SessionId::try_from(session_cookie.value()) {
            if let Some(session) = app_state
                .web_sessions
                .get_session(session_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            {
                req.extensions_mut().insert(session);
                return Ok((jar, next.run(req).await));
            }
        }
    }

    let session = app_state
        .web_sessions
        .new_session()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("created new session {}", session.id);

    let session_id = session.id;
    req.extensions_mut().insert(session);

    let cookie = Cookie::build(("Session", session_id.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .expires(Expiration::Session)
        .max_age(Duration::days(7))
        .build();
    Ok((jar.add(cookie), next.run(req).await))
}
