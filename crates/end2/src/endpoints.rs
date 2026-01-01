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

use crate::{AppState, DirectMessageLink, RoomId, Session, UserName, handle_socket};

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    name: Option<&'a UserName>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login<'a> {
    name: Option<&'a UserName>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct Register<'a> {
    name: Option<&'a UserName>,
}

#[derive(Template)]
#[template(path = "messages.html")]
struct Messages<'a> {
    name: Option<&'a UserName>,
    messages: Vec<DirectMessageLink>,
}

#[derive(Template)]
#[template(path = "dm.html")]
struct DirectMessage<'a> {
    name: Option<&'a UserName>,
}

#[tracing::instrument]
pub async fn index(session: Option<Session>) -> Result<impl IntoResponse, StatusCode> {
    let index = Index {
        name: session.as_ref().map(|s| s.username()),
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

pub async fn direct_messages(
    State(app_state): State<AppState>,
    session: Option<Session>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(session) = session {
        let dms = app_state.get_direct_messages(&session).await.unwrap();

        let messages = Messages {
            name: Some(session.username()),
            messages: dms,
        };

        Ok(Html(
            messages
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        ))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Deserialize)]
pub struct MessageRequest {
    pub recipient: UserName,
}

pub async fn message_request(
    State(app_state): State<AppState>,
    session: Option<Session>,
    Form(MessageRequest { recipient }): Form<MessageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(session) = session {
        app_state
            .create_dm(&session, recipient)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Redirect::to("/dms").into_response())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Deserialize)]
pub struct SessionParams {
    pub room_id: RoomId,
}

pub async fn direct_message(
    State(app_state): State<AppState>,
    Path(SessionParams { room_id }): Path<SessionParams>,
    session: Option<Session>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(session) = session {
        let dm = DirectMessage {
            name: Some(session.username()),
        };

        Ok(Html(
            dm
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        ))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn direct_message_ws(
    State(app_state): State<AppState>,
    Path(SessionParams { room_id }): Path<SessionParams>,
    session: Option<Session>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(session) = session {
        Ok(ws.on_upgrade(move |socket| handle_socket(socket, session, room_id, app_state)))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
