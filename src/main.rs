use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use base64::prelude::*;
use rand_core::OsRng;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use p256::ecdh::diffie_hellman;
use p256::ecdsa::SigningKey;
use p256::{PublicKey, SecretKey};
use hkdf::Hkdf;
use sha2::Sha256;

#[derive(Clone, Serialize)]
struct Peer {
    address: String,
    public_key: PublicKey,
}

#[derive(Clone)]
struct AppState {
    secret: SecretKey,
    signing: SigningKey,
    peers: Arc<RwLock<Vec<Peer>>>,
    port: u16,
}

impl AppState {
    fn port(&self) -> u16 {
        self.port
    }
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("Server running on http://localhost:{}", port);

    // let alice_private = SecretKey::random(&mut OsRng);
    // let alice_sign = SigningKey::random(&mut OsRng);
    // let alice_verify = alice_sign.verifying_key();
    // let alice_public = alice_private.public_key();

    // let bob_private = SecretKey::random(&mut OsRng);
    // let bob_public = bob_private.public_key();

    // let alice_bytes = alice_public.to_sec1_bytes();
    // let bob_bytes = bob_public.to_sec1_bytes();

    // let ab_public = PublicKey::from_sec1_bytes(&bob_bytes).ok().unwrap();
    // let ba_public = PublicKey::from_sec1_bytes(&alice_bytes).ok().unwrap();

    // let alice_secret = diffie_hellman(alice_private.to_nonzero_scalar(), ab_public.as_affine());

    // let ikm = alice_secret.raw_secret_bytes();
    // let salt = b"my-protocol-salt-v1";
    // let info = b"info";
    // let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &ikm);
    // let mut okm = [0u8; 33];
    // hk.expand(info, &mut okm).expect("expand");

    // println!("{}", BASE64_STANDARD.encode(okm));

    // let bob_secret = diffie_hellman(bob_private.to_nonzero_scalar(), ba_public.as_affine());

    // let ikm = bob_secret.raw_secret_bytes();
    // let salt = b"my-protocol-salt-v1";
    // let info = b"info";
    // let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &ikm);
    // let mut okm = [0u8; 33];
    // hk.expand(info, &mut okm).expect("expand");

    // println!("{}", BASE64_STANDARD.encode(okm));

    let shared_state = AppState {
        secret: SecretKey::random(&mut OsRng),
        signing: SigningKey::random(&mut OsRng),
        peers: Arc::new(RwLock::new(vec![])),
        port,
    };

    let app = Router::new()
        .with_state(shared_state);

    axum::serve(listener, app).await.unwrap();
}

// gossip-based public-key verification:
// peer A connects to peer B
// peer A connects to peer C
// peer A asks peer C what peer B's public key is

// response has four cases:
// peer C is a bad actor and gives a correct public key (tells truth)
// peer C is a bad actor and gives an incorrect public key (lies)
// peer C is a good actor and gives a correct public key
// peer C is a good actor and gives an incorrect public key

// an adversary can either have a bunch of peers that lie about B's
// public key, or can trick a bunch of good peers about B's public key
// threshold can be 25% trick, 25% lie
//
// peer B has 50 peers connected, need to make at least 25 or 26 fake peers
// to lie, and trick at least 25 or 26 newly connecting peers to connecting
// to adversary instead, then gossip will start pointing to adversary rather
// than peer B 