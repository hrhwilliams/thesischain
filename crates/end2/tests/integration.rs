mod common;

use anyhow::Result;
use common::{ApiClient, spawn_app};
use reqwest::StatusCode;

#[tokio::test]
async fn integration() -> Result<()> {
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
