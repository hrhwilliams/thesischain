mod common;

use anyhow::Result;
use common::ApiClient;
use diesel::{PgConnection, r2d2::ConnectionManager};
use end2::{App, OAuthHandler};
use tokio::net::TcpListener;

#[tokio::test]
async fn integration() -> Result<()> {
    // Create app
    let ip = "localhost";

    let port = 8081;

    let client_id = String::new();
    let client_secret = String::new();
    let redirect = String::new();

    let oauth = OAuthHandler::new(client_id, client_secret, redirect);

    let database_url = "postgres://postgres@localhost/postgres".to_string();
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to Postgres");

    let app = App::new(oauth, pool);

    let listener = TcpListener::bind((ip, port)).await.expect("TcpListener");

    let app_task = tokio::spawn(async move { app.run(listener).await });

    // Create users
    let mut client1 = ApiClient::new("maki1", "abc", None).await?;
    let mut client2 = ApiClient::new("maki2", "abc", None).await?;

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

    let history = client1.get_history(channel1_id.channel_id).await?;

    println!("Client 1 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }

    // Client 2 lists the channels they are in, selects the first one, and reads the history
    let channels2 = client2.channels().await?;
    let channel2_id = channels2.first().unwrap();
    let channel2_info = client2.get_channel_participants(channel2_id).await?;

    let history = client2.get_history(channel2_id.channel_id).await?;

    println!("Client 2 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    // Client 2 responds
    client2.send_message(&channel1_info, "hello!").await?;

    // Client 1 reads the history
    let history = client1.get_history(channel1_id.channel_id).await?;

    println!("Client 1 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }

    // Client 2 reads the history again
    let history = client2.get_history(channel2_id.channel_id).await?;

    println!("Client 2 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    // Client 2 sends a message
    client2.send_message(&channel2_info, "hello again!").await?;

    // Client 1 sends a message
    client1.send_message(&channel1_info, "how are you?").await?;

    // Client 1 reads the history
    let history = client1.get_history(channel2_id.channel_id).await?;

    println!("---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    // Client 2 reads the history again
    let history = client2.get_history(channel2_id.channel_id).await?;

    println!("Client 2 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    Ok(())
}
