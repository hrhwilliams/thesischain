use crate::startup::KeyChainData;
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::header::{self, ContentType},
    web,
};
use serde::{Deserialize, Serialize};

use crate::chain::Node;
use crate::peer::Peer;

pub async fn index(app: web::Data<KeyChainData>) -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
</head>
<body>
  <h1>Peer {}</h1>
  <a href="/get_peers">Peer list</a>
  <br>
  <a href="/chain">Chain</a>
</body>
</html>
"#,
            app.port()
        ))
}

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound()
        .content_type(ContentType::html())
        .body(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
</head>
<body>
  <h1>Not found</h1>
  <ul>
    <li><a href="/">Index</a></li>
    <li><a href="/get_peers">Peer list</a></li>
    <li><a href="/chain">Chain</a></li>
  </ul>
</body>
</html>
"#,
        )
}

#[derive(Serialize, Deserialize)]
pub struct PeerMessage {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct PeerList {
    pub peers: Vec<Peer>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockMessage {
    pub block: Node,
}

#[derive(Deserialize)]
pub struct ManualPeerForm {
    port: u16,
}

pub async fn add_peer(req: web::Json<PeerMessage>, app: web::Data<KeyChainData>) -> impl Responder {
    app.get_ref()
        .insert_peers(vec![Peer {
            address: req.address.clone(),
        }])
        .await;
    HttpResponse::Ok()
}

pub async fn add_peer_form() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(
        r#"
            <!doctype html>
            <html>
                <head>
                    <title>Add Peer</title>
                </head>
                <body>
                    <h1>Add a new peer</h1>
                    <form action="/add_peer_manual" method="post">
                        <label for="port">Peer Port:</label>
                        <input type="text" id="port" name="port">
                        <button type="submit">Connect</button>
                    </form>
                </body>
            </html>
            "#,
    )
}

pub async fn add_peer_manual(
    form: web::Form<ManualPeerForm>,
    data: web::Data<KeyChainData>,
) -> impl Responder {
    let client = reqwest::Client::new();
    let new_peer_address = format!("http://127.0.0.1:{}", form.port);
    let self_address = format!("http://127.0.0.1:{}", data.port());

    // Tell the new peer about us
    let res = client
        .post(format!("{}/add_peer", new_peer_address))
        .json(&PeerMessage {
            address: self_address,
        })
        .send()
        .await;

    match res {
        Ok(response) if response.status().is_success() => {
            // Add the new peer to our own list
            data.get_ref()
                .insert_peers(vec![Peer {
                    address: new_peer_address,
                }])
                .await;
            // Redirect back to the home page
            HttpResponse::SeeOther()
                .append_header((header::LOCATION, "/"))
                .finish()
        }
        _ => HttpResponse::InternalServerError()
            .body(format!("Failed to add peer at port {}", form.port)),
    }
}

pub async fn get_peers(req: HttpRequest, app: web::Data<KeyChainData>) -> impl Responder {
    let wants_html = req
        .headers()
        .get(actix_web::http::header::ACCEPT)
        .and_then(|val| val.to_str().ok())
        .map_or(false, |h| h.contains("text/html"));
    let peers = app.peers.read().await;

    if wants_html {
        let list_items: String = peers
            .iter()
            .map(|p| format!("<li><a href={}/get_peers>{}</a></li>", p.address, p.address))
            .collect();

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
</head>
<body>
  <h1>Known peers ({})</h1>
  <p>
  <a href="/">Index</a> | <a href="/chain">Chain</a> | <a href="/add_peer">Add Peer</a>
  </p>
  <br>
  <ul>
    {}
  </ul>
</body>
</html>"#,
            peers.len(),
            list_items
        );

        HttpResponse::Ok().body(html)
    } else {
        let mut peer_list: Vec<Peer> = vec![];

        for peer in peers.iter() {
            peer_list.push(peer.clone());
        }

        log::info!("Responding to get peers request");

        HttpResponse::Ok().body(
            serde_json::to_string(&PeerList { peers: peer_list })
                .expect("Failed to serialize to JSON"),
        )
    }
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn get_chain(app: web::Data<KeyChainData>) -> impl Responder {
    let chain = app.chain.read().await;
    HttpResponse::Ok().json(&*chain)
}

pub async fn add_block(
    req: web::Json<BlockMessage>,
    app: web::Data<KeyChainData>,
) -> impl Responder {
    let new_block = req.into_inner().block;
    let mut chain = app.chain.write().await;
    let current_head = chain.last().unwrap();

    if new_block.parent == Some(current_head.hash) {
        log::info!("Added new block to chain from peer.");
        chain.push(new_block);
        HttpResponse::Ok().finish()
    } else {
        log::warn!("Received block with invalid parent.");
        HttpResponse::BadRequest().body("Invalid parent block")
    }
}
