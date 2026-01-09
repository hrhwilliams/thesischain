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
use uuid::Uuid;

use crate::AppState;

pub async fn create_session(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let (mut parts, body) = req.into_parts();
    let jar = CookieJar::from_request_parts(&mut parts, &state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // let api_route = parts.uri.path().starts_with("/api");
    let req = Request::from_parts(parts, body);

    if let Some(session_cookie) = jar.get("Session") {
        let session_id = Uuid::parse_str(session_cookie.value())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if state
            .get_session(session_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .is_some()
        {
            return Ok((jar, next.run(req).await));
        }
    }

    let session = state
        .new_session()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build(("Session", session.id.to_string()))
        // .http_only(true)
        // .secure(true)
        .same_site(SameSite::None)
        .path("/")
        .expires(Expiration::Session)
        .max_age(Duration::days(7))
        .build();
    Ok((jar.add(cookie), next.run(req).await))
}
