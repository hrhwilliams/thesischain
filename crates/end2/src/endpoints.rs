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
use futures::TryFutureExt;
use serde::Deserialize;

use crate::{AppState, LoginError, RegistrationError, Room, User};

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    name: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login {
    name: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct Register {
    name: Option<String>,
}

#[derive(Template)]
#[template(path = "messages.html")]
struct Messages {
    name: Option<String>,
    rooms: Vec<Room>,
}

#[derive(Template)]
#[template(path = "dm.html")]
struct DirectMessage {
    name: Option<String>,
}

#[tracing::instrument]
pub async fn index(user: Option<User>) -> Result<impl IntoResponse, StatusCode> {
    let index = if let Some(user) = user {
        Index {
            name: Some(user.username),
        }
    } else {
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
    pub username: String,
    pub password: String,
}

#[tracing::instrument]
pub async fn display_login_form(user: Option<User>) -> Result<Response, StatusCode> {
    if user.is_some() {
        tracing::info!("User already logged in found; redirecting");
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
    user: Option<User>,
    jar: CookieJar,
    Form(LoginForm { username, password }): Form<LoginForm>,
) -> Result<Response, StatusCode> {
    if user.is_some() {
        tracing::info!("User already logged in found; redirecting");
        Ok(Redirect::to("/").into_response())
    } else {
        match app_state
            .validate_password_and_get_user(username, password)
            .await
        {
            Ok(user) => {
                let session = app_state
                    .create_session_for_user(user)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                let cookie = Cookie::build(("Session", session.id.to_string()))
                    .expires(Expiration::Session)
                    .same_site(SameSite::Strict)
                    .build();
                Ok((jar.add(cookie), Redirect::to("/")).into_response())
            }
            Err(LoginError::UserNotFound) | Err(LoginError::InvalidPassword) => {
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(LoginError::System(e)) => {
                tracing::error!("{:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[tracing::instrument(skip(app_state))]
pub async fn logout(
    State(app_state): State<AppState>,
    user: Option<User>,
    jar: CookieJar,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Logging user out");
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    app_state
        .delete_user_session(user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((jar.remove(Cookie::from("Session")), Redirect::to("/")))
}

#[derive(Deserialize)]
pub struct SignUp {
    pub username: String,
    pub password: String,
    pub confirmation: String,
}

#[tracing::instrument]
pub async fn register_form(user: Option<User>) -> Result<impl IntoResponse, StatusCode> {
    if user.is_some() {
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
    jar: CookieJar,
    Form(SignUp {
        username,
        password,
        confirmation,
    }): Form<SignUp>,
) -> Result<impl IntoResponse, StatusCode> {
    match app_state
        .register_user(username, password, confirmation)
        .await
    {
        Ok(user) => {
            let session = app_state
                .create_session_for_user(user)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let cookie = Cookie::build(("Session", session.id.to_string()))
                .expires(Expiration::Session)
                .same_site(SameSite::Strict)
                .build();
            Ok((jar.add(cookie), Redirect::to("/")).into_response())
        }
        Err(RegistrationError::InvalidUsername)
        | Err(RegistrationError::InvalidPassword)
        | Err(RegistrationError::PasswordMismatch) => Err(StatusCode::BAD_REQUEST),
        Err(RegistrationError::UsernameTaken) => Err(StatusCode::CONFLICT),
        Err(RegistrationError::System(e)) => {
            tracing::error!("{:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn direct_messages(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;
    let username = user.username.clone();

    let users_rooms = app_state
        .get_rooms(user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let messages = Messages {
        name: Some(username),
        rooms: users_rooms,
    };

    Ok(Html(
        messages
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

// #[derive(Deserialize)]
// pub struct MessageRequest {
//     pub recipient: UserName,
// }

// pub async fn message_request(
//     State(app_state): State<AppState>,
//     user: Option<User>,
//     Form(MessageRequest { recipient }): Form<MessageRequest>,
// ) -> Result<impl IntoResponse, StatusCode> {
//     if let Some(session) = session {
//         app_state
//             .create_dm(&session, recipient)
//             .await
//             .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//         Ok(Redirect::to("/dms").into_response())
//     } else {
//         Err(StatusCode::UNAUTHORIZED)
//     }
// }

// #[derive(Deserialize)]
// pub struct SessionParams {
//     pub room_id: RoomId,
// }

// pub async fn direct_message(
//     State(app_state): State<AppState>,
//     Path(SessionParams { room_id }): Path<SessionParams>,
//     user: Option<User>,
// ) -> Result<impl IntoResponse, StatusCode> {
//     if let Some(session) = session {
//         let dm = DirectMessage {
//             name: Some(session.username()),
//         };

//         Ok(Html(
//             dm.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
//         ))
//     } else {
//         Err(StatusCode::UNAUTHORIZED)
//     }
// }

// pub async fn direct_message_ws(
//     State(app_state): State<AppState>,
//     Path(SessionParams { room_id }): Path<SessionParams>,
//     user: Option<User>,
//     ws: WebSocketUpgrade,
// ) -> Result<impl IntoResponse, StatusCode> {
//     if let Some(session) = session {
//         Ok(ws.on_upgrade(move |socket| handle_socket(socket, session, room_id, app_state)))
//     } else {
//         Err(StatusCode::UNAUTHORIZED)
//     }
// }
