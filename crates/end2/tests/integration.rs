#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    dead_code,
)]

mod common;

use anyhow::Result;
use common::{ApiClient, spawn_app, spawn_app_with_chain, spawn_app_with_miners};
use miner::{IdentityAttestation, Transaction, sign_attestation, sign_transaction};
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Duration;

#[tokio::test]
async fn test_centralized_service_e2ee() -> Result<()> {
    let port = spawn_app().await;

    // Create users
    let mut client1 = ApiClient::with_port("maki1", "abc", None, port).await?;
    let mut client2 = ApiClient::with_port("maki2", "abc", None, port).await?;

    // Upload one-time keys for users
    client1.upload_otks().await?;
    client2.upload_otks().await?;

    // Client 1 creates a channel to message Client 2 through
    client1.create_channel("maki2").await?;

    // Client 2 lists the channels they are in and selects the first one
    let channels1 = client1.channels().await?;
    let channel1_id = channels1.first().unwrap();
    let channel1_info = client1.get_channel_participants(channel1_id).await?;

    // Client 1 sends a message to the channel
    client1.send_message(&channel1_info, "hello").await?;

    // Client 2 reads history — the message was encrypted for client2's device
    let channels2 = client2.channels().await?;
    let channel2_id = channels2.first().unwrap();
    let _channel2_info = client2.get_channel_participants(channel2_id).await?;

    let history = client2.get_history(channel2_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "hello");

    // Client 2 responds
    client2.send_message(&channel1_info, "hello!").await?;

    // Client 1 reads the history — now they can see client2's reply (encrypted for client1's device)
    let history = client1.get_history(channel1_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "hello!");

    // Back-and-forth: client1 sends again
    client1.send_message(&channel1_info, "how are you?").await?;

    // Client 2 reads — should see the new message
    let history = client2.get_history(channel2_id.channel_id).await?;
    assert_eq!(history.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_invalid_credentials() -> Result<()> {
    let port = spawn_app().await;

    // Register a user
    let _client = ApiClient::with_port("validuser", "correctpass", None, port).await?;

    // Try to login with wrong password
    let http_client = reqwest::Client::new();
    let response = http_client
        .post(format!("http://localhost:{port}/api/auth/login"))
        .json(&serde_json::json!({
            "username": "validuser",
            "password": "wrongpass",
        }))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_unauthorized_channel_access() -> Result<()> {
    let port = spawn_app().await;

    // Create three users
    let mut client1 = ApiClient::with_port("alice", "pass", None, port).await?;
    let mut client2 = ApiClient::with_port("bob", "pass", None, port).await?;
    let client3 = ApiClient::with_port("carol", "pass", None, port).await?;

    client1.upload_otks().await?;
    client2.upload_otks().await?;

    // Alice creates a channel with Bob
    client1.create_channel("bob").await?;
    let channels = client1.channels().await?;
    let channel_id = channels.first().unwrap().channel_id;

    // Carol (not in channel) tries to read history — should get an error
    let response = client3
        .raw_get(&format!(
            "/channel/{}/history?device={}",
            channel_id,
            uuid::Uuid::nil()
        ))
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "non-participant should not access channel history"
    );

    Ok(())
}

#[tokio::test]
async fn test_message_back_and_forth() -> Result<()> {
    let port = spawn_app().await;

    let mut client1 = ApiClient::with_port("order1", "pass", None, port).await?;
    let mut client2 = ApiClient::with_port("order2", "pass", None, port).await?;

    client1.upload_otks().await?;
    client2.upload_otks().await?;

    client1.create_channel("order2").await?;

    let channels1 = client1.channels().await?;
    let channel1_id = channels1.first().unwrap();
    let channel1_info = client1.get_channel_participants(channel1_id).await?;

    let channels2 = client2.channels().await?;
    let channel2_id = channels2.first().unwrap();

    // Round 1: client1 → client2
    client1.send_message(&channel1_info, "first").await?;

    let history = client2.get_history(channel2_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "first");

    // Round 2: client2 → client1 (ratchet step allows sending)
    client2.send_message(&channel1_info, "second").await?;

    let history = client1.get_history(channel1_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "second");

    // Round 3: client1 → client2 again
    client1.send_message(&channel1_info, "third").await?;

    let history = client2.get_history(channel2_id.channel_id).await?;
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].plaintext, "third");

    Ok(())
}

/// E2EE messaging where clients register their device keys on the blockchain
/// via the miner HTTP API, and the backend reads keys from the same chain.
/// Requires a backend-signed identity attestation for each device registration.
#[tokio::test]
async fn test_chain_backed_e2ee() -> Result<()> {
    let env = spawn_app_with_chain().await;
    let miner_url = format!("http://localhost:{}", env.miner_port);
    let backend_url = format!("http://localhost:{}/api", env.backend_port);
    let http = reqwest::Client::new();

    // Phase 1: Each client gets a backend attestation, then registers on-chain
    for (i, tu) in env.test_users.iter().enumerate() {
        // Get identity attestation from backend (requires auth)
        let attest_resp = http
            .post(format!("{backend_url}/auth/attest"))
            .basic_auth(&tu.username, Some(&tu.password))
            .json(&serde_json::json!({ "device_id": tu.device.id() }))
            .send()
            .await?;
        assert_eq!(
            attest_resp.status(),
            StatusCode::OK,
            "get attestation for user {i}"
        );

        #[derive(Deserialize)]
        struct AttestResponse {
            attestation: IdentityAttestation,
        }
        let attestation = attest_resp.json::<AttestResponse>().await?.attestation;

        let tx = sign_transaction(
            Transaction::RegisterDevice {
                user_id: tu.user_id,
                device_id: tu.device.id(),
                ed25519: tu.chain_signing_key.verifying_key().to_bytes(),
                x25519: tu.device.x25519_public_key_bytes(),
                attestation,
            },
            i as u64, // nonce
            &tu.chain_signing_key,
        )
        .expect("sign tx");

        let resp = http
            .post(format!("{miner_url}/tx"))
            .json(&tx)
            .send()
            .await?;
        assert_eq!(resp.status(), StatusCode::ACCEPTED, "submit tx for user {i}");
    }

    // Phase 2: Trigger block production — miner mines a block with the pending txs
    let resp = http
        .post(format!("{miner_url}/mine"))
        .send()
        .await?;
    assert_eq!(resp.status(), StatusCode::OK, "mine block");

    // Verify devices are now on-chain
    for tu in &env.test_users {
        let resp = http
            .get(format!("{miner_url}/device/{}", tu.device.id()))
            .send()
            .await?;
        assert_eq!(resp.status(), StatusCode::OK, "device {} on chain", tu.device.id());
    }

    // Phase 3: Standard E2EE flow through the backend (which reads keys from chain)
    let mut test_users = env.test_users;
    let user2 = test_users.remove(1);
    let user1 = test_users.remove(0);
    let user2_username = user2.username.clone();

    let mut client1 = ApiClient::preconfigured(
        &user1.username, &user1.password, user1.user_id, user1.device, env.backend_port,
    );
    let mut client2 = ApiClient::preconfigured(
        &user2.username, &user2.password, user2.user_id, user2.device, env.backend_port,
    );

    // OTKs are still DB-backed
    client1.upload_otks().await?;
    client2.upload_otks().await?;

    // Create channel and exchange messages
    client1.create_channel(&user2_username).await?;

    let channels1 = client1.channels().await?;
    let channel1_id = channels1.first().unwrap();
    let channel1_info = client1.get_channel_participants(channel1_id).await?;

    client1
        .send_message(&channel1_info, "hello from chain")
        .await?;

    let channels2 = client2.channels().await?;
    let channel2_id = channels2.first().unwrap();

    let history = client2.get_history(channel2_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "hello from chain");

    // Client 2 responds
    client2
        .send_message(&channel1_info, "chain reply")
        .await?;

    let history = client1.get_history(channel1_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "chain reply");

    Ok(())
}

/// Submitting a RegisterDevice transaction with a fake/invalid attestation
/// should be rejected by the miner.
#[tokio::test]
async fn test_register_without_attestation_rejected() -> Result<()> {
    let env = spawn_app_with_chain().await;
    let miner_url = format!("http://localhost:{}", env.miner_port);
    let http = reqwest::Client::new();

    let tu = &env.test_users[0];

    // Create a fake attestation signed by a random key (not the backend key)
    let fake_key = ed25519_dalek::SigningKey::generate(&mut rand_core::OsRng);
    let fake_attestation = sign_attestation(
        tu.user_id,
        tu.device.id(),
        0,
        &fake_key,
    )
    .expect("sign fake attestation");

    let tx = sign_transaction(
        Transaction::RegisterDevice {
            user_id: tu.user_id,
            device_id: tu.device.id(),
            ed25519: tu.chain_signing_key.verifying_key().to_bytes(),
            x25519: tu.device.x25519_public_key_bytes(),
            attestation: fake_attestation,
        },
        0,
        &tu.chain_signing_key,
    )
    .expect("sign tx");

    let resp = http
        .post(format!("{miner_url}/tx"))
        .json(&tx)
        .send()
        .await?;

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "tx with invalid attestation should be rejected"
    );

    Ok(())
}

/// Spawns 3 P2P miner nodes and verifies that transactions submitted to
/// different miners propagate to all via gossipsub consensus, then runs
/// E2EE messaging through the backend.
#[tokio::test]
async fn test_3_miner_consensus() -> Result<()> {
    let env = spawn_app_with_miners(3).await;
    let backend_url = format!("http://localhost:{}/api", env.backend_port);
    let http = reqwest::Client::new();

    // Phase 1: Each user gets attestation from backend, registers on a DIFFERENT miner
    for (i, tu) in env.test_users.iter().enumerate() {
        let attest_resp = http
            .post(format!("{backend_url}/auth/attest"))
            .basic_auth(&tu.username, Some(&tu.password))
            .json(&serde_json::json!({ "device_id": tu.device.id() }))
            .send()
            .await?;
        assert_eq!(
            attest_resp.status(),
            StatusCode::OK,
            "get attestation for user {i}"
        );

        #[derive(Deserialize)]
        struct AttestResponse {
            attestation: IdentityAttestation,
        }
        let attestation = attest_resp.json::<AttestResponse>().await?.attestation;

        let tx = sign_transaction(
            Transaction::RegisterDevice {
                user_id: tu.user_id,
                device_id: tu.device.id(),
                ed25519: tu.chain_signing_key.verifying_key().to_bytes(),
                x25519: tu.device.x25519_public_key_bytes(),
                attestation,
            },
            i as u64,
            &tu.chain_signing_key,
        )
        .expect("sign tx");

        // Submit to miner i (different miner for each user)
        let miner_url = format!(
            "http://localhost:{}",
            env.miner_ports[i % env.miner_ports.len()]
        );
        let resp = http
            .post(format!("{miner_url}/tx"))
            .json(&tx)
            .send()
            .await?;
        assert_eq!(
            resp.status(),
            StatusCode::ACCEPTED,
            "submit tx for user {i} to miner {}", i % env.miner_ports.len()
        );
    }

    // Phase 2: Poll until all miners have all devices (block production + gossip propagation)
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let mut all_found = true;
        for miner_port in &env.miner_ports {
            for tu in &env.test_users {
                let resp = http
                    .get(format!(
                        "http://localhost:{miner_port}/device/{}",
                        tu.device.id()
                    ))
                    .send()
                    .await?;
                if resp.status() != StatusCode::OK {
                    all_found = false;
                    break;
                }
            }
            if !all_found {
                break;
            }
        }
        if all_found {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for all miners to have all devices"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Phase 4: E2EE messaging through backend (reads keys from miner 0's chain)
    let mut test_users = env.test_users;
    let user2 = test_users.remove(1);
    let user1 = test_users.remove(0);
    let user2_username = user2.username.clone();

    let mut client1 = ApiClient::preconfigured(
        &user1.username,
        &user1.password,
        user1.user_id,
        user1.device,
        env.backend_port,
    );
    let mut client2 = ApiClient::preconfigured(
        &user2.username,
        &user2.password,
        user2.user_id,
        user2.device,
        env.backend_port,
    );

    client1.upload_otks().await?;
    client2.upload_otks().await?;

    client1.create_channel(&user2_username).await?;

    let channels1 = client1.channels().await?;
    let channel1_id = channels1.first().unwrap();
    let channel1_info = client1.get_channel_participants(channel1_id).await?;

    client1
        .send_message(&channel1_info, "hello from consensus")
        .await?;

    let channels2 = client2.channels().await?;
    let channel2_id = channels2.first().unwrap();

    let history = client2.get_history(channel2_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "hello from consensus");

    client2
        .send_message(&channel1_info, "consensus reply")
        .await?;

    let history = client1.get_history(channel1_id.channel_id).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].plaintext, "consensus reply");

    Ok(())
}
