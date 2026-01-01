use askama::Template;
use axum::{
    Form,
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, Expiration, SameSite},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, Session, UserName, handle_socket};

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    name: Option<UserName>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login {
    name: Option<UserName>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct Register {
    name: Option<UserName>,
}

#[tracing::instrument]
pub async fn index(session: Option<Session>) -> Result<impl IntoResponse, StatusCode> {
    let index = if let Some(session) = session {
        tracing::info!("Session found");
        Index {
            name: Some(session.user_info.username),
        }
    } else {
        tracing::info!("No session");
        Index { name: None }
    };

    Ok(Html(
        index
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: UserName,
    pub password: String,
}

#[tracing::instrument]
pub async fn login_form(session: Option<Session>) -> Result<Response, StatusCode> {
    if session.is_some() {
        tracing::info!("Session found; redirecting");
        Ok(Redirect::to("/").into_response())
    } else {
        let login = Login { name: None };

        Ok(Html(
            login
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        )
        .into_response())
    }
}

#[tracing::instrument(skip(app_state))]
pub async fn login(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Form(LoginForm { username, password }): Form<LoginForm>,
) -> Result<Response, StatusCode> {
    if let Some(session) = app_state.create_session(username, &password).await {
        tracing::info!("Logging user in with session id {:?}", session.session_id());
        let cookie = Cookie::build(("Session", session.session_id().0))
            .expires(Expiration::Session)
            .build();

        Ok((jar.add(cookie), Redirect::to("/")).into_response())
    } else {
        tracing::info!("Invalid password or username");
        Ok(Redirect::to("/login").into_response())
    }
}

#[tracing::instrument]
pub async fn logout(jar: CookieJar) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Logging user out");
    Ok((jar.remove(Cookie::from("Session")), Redirect::to("/")))
}

#[derive(Deserialize)]
pub struct SignUp {
    pub username: UserName,
    pub password: String,
    pub confirm: String,
}

#[tracing::instrument]
pub async fn register_form(session: Option<Session>) -> Result<impl IntoResponse, StatusCode> {
    if session.is_some() {
        tracing::info!("Session found; redirecting");
        Ok(Redirect::to("/").into_response())
    } else {
        let register = Register { name: None };

        Ok(Html(
            register
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        )
        .into_response())
    }
}

#[tracing::instrument(skip(app_state))]
pub async fn register(
    State(app_state): State<AppState>,
    Form(SignUp {
        username,
        password,
        confirm,
    }): Form<SignUp>,
) -> Result<impl IntoResponse, StatusCode> {
    let success = app_state
        .register_user(username, &password, &confirm)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if success {
        tracing::info!("User registered");
        Ok(Redirect::to("/login"))
    } else {
        tracing::info!("Failed to register user");
        Ok(Redirect::to("/register"))
    }
}

#[derive(Deserialize)]
pub struct SessionParams {
    pub session_id: Uuid,
}

pub async fn session(
    State(app_state): State<AppState>,
    Path(SessionParams { session_id }): Path<SessionParams>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, session_id, app_state)))
}
