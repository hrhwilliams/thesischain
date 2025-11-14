use std::{env, io::Write};

use anyhow::Result;
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use vodozemac::{
    Curve25519PublicKey,
    olm::{
        Account, InboundCreationResult, OlmMessage, OneTimeKeyGenerationResult, Session,
        SessionConfig,
    },
};

#[derive(Serialize, Deserialize)]
pub enum Message {
    IdentityKey(Curve25519PublicKey),
    OneTimeKey(Curve25519PublicKey),
    Message(OlmMessage),
}

impl Message {
    pub fn identity_key(account: &Account) -> Message {
        Message::IdentityKey(account.curve25519_key())
    }

    pub fn one_time_key(account: &Account) -> Message {
        Message::OneTimeKey(*account.one_time_keys().values().next().unwrap())
    }

    pub fn message(session: &mut Session, message: impl AsRef<[u8]>) -> Self {
        Message::Message(session.encrypt(message))
    }

    pub fn bincode(self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }
}

// #[derive(Serialize, Deserialize)]
// struct Message {
//     pub message_type: MessageType,
//     pub message: OlmMessage,
// }

async fn handle_server(port: &str) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port.parse()?)).await?;
    let (stream, _addr) = listener.accept().await?;
    let (mut stream_read, mut stream_write) = stream.into_split();

    let mut stdin_reader = BufReader::new(io::stdin());
    let mut stdin_line = String::new();

    let mut account = Account::new();
    account.generate_one_time_keys(1);

    let mut buffer = vec![0u8; 2048];

    let client_id_key: Curve25519PublicKey;
    let client_otk: Curve25519PublicKey;


    stream_read.read(&mut buffer).await?;
    let (message, _): (Message, usize) =
        bincode::serde::borrow_decode_from_slice(&buffer, bincode::config::standard())?;

    match message {
        Message::IdentityKey(m) => {
            client_id_key = m;
        }
        _ => panic!("Client didn't send ID key"),
    }

    stream_write.write(&Message::identity_key(&account).bincode()?).await?;

    stream_read.read(&mut buffer).await?;
    let (message, _): (Message, usize) =
        bincode::serde::borrow_decode_from_slice(&buffer, bincode::config::standard())?;

    match message {
        Message::OneTimeKey(m) => {
            client_otk = m;
        }
        _ => panic!("Client didn't send OTK key"),
    }

    stream_write.write(&Message::one_time_key(&account).bincode()?).await?;

    let mut session =
        account.create_outbound_session(SessionConfig::version_2(), client_id_key, client_otk);

    stream_read.read(&mut buffer).await?;
    let (message, _): (Message, usize) =
        bincode::serde::borrow_decode_from_slice(&buffer, bincode::config::standard())?;

    match message {
        Message::Message(m) => match m {
            OlmMessage::PreKey(m) => {
                let result = account.create_inbound_session(client_id_key, &m)?;
                println!("{}", String::from_utf8_lossy(&result.plaintext))
            }
            _ => panic!("Client sent normal message"),
        },
        _ => panic!("Client didn't send message"),
    }

    stream_write.write(&Message::message(&mut session, "Hello, client!").bincode()?).await?;

    // loop {
    //     tokio::select! {
    //         ready = stream_read.readable() => {
    //             println!("huh");
    //             match ready {
    //                 Ok(()) => match stream_read.try_read(&mut buffer) {
    //                     Ok(0) => {
    //                         println!("connection closed by peer");
    //                         return Ok(());
    //                     }
    //                     Ok(n) => {
    //                         println!("received {} bytes: {}", n, BASE64_STANDARD.encode(&buffer[..n]));

    //                         let (message, bytes): (Message, usize) = bincode::serde::borrow_decode_from_slice(&buffer[..n], bincode::config::standard())?;
    //                         match message {
    //                             Message::IdentityKey(m) => {},
    //                             Message::OneTimeKey(m) => {},
    //                             Message::Message(m) => {},
    //                         }
    //                     }
    //                     Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
    //                         continue;
    //                     }
    //                     Err(err) => {
    //                         eprintln!("failed to read from socket: {err}");
    //                         return Ok(());
    //                     }
    //                 },
    //                 Err(err) => {
    //                     eprintln!("socket readable() error: {err}");
    //                     return Ok(());
    //                 }
    //             }
    //         }
    //         stdin_res = stdin_reader.read_line(&mut stdin_line) => {
    //             match stdin_res {
    //                 Ok(0) => {
    //                     println!("stdin closed");
    //                     return Ok(());
    //                 }
    //                 Ok(_) => {
    //                     // let msg = Message::new(&mut session, stdin_line.as_bytes());
    //                     let msg = Message::message(&mut session, stdin_line.as_bytes());
    //                     let msg_bytes = bincode::serde::encode_to_vec(msg, bincode::config::standard())?;
    //                     if let Err(err) = stream_write.write_all(&msg_bytes).await {
    //                         eprintln!("failed to write to socket: {err}");
    //                         return Ok(());
    //                     }
    //                     stdin_line.clear();
    //                 }
    //                 Err(err) => {
    //                     eprintln!("failed to read stdin: {err}");
    //                     return Ok(());
    //                 }
    //             }
    //         }
    //     }
    // }

    Ok(())
}

async fn handle_client(port: &str) -> Result<()> {
    let stream = TcpStream::connect(("127.0.0.1", port.parse()?)).await?;
    let (mut stream_read, mut stream_write) = stream.into_split();

    let mut stdin_reader = BufReader::new(io::stdin());
    let mut stdin_line = String::new();

    let mut account = Account::new();
    account.generate_one_time_keys(1);

    let mut buffer = vec![0u8; 2048];

    let server_id_key: Curve25519PublicKey;
    let server_otk: Curve25519PublicKey;


    stream_write.write(&Message::identity_key(&account).bincode()?).await?;

    stream_read.read(&mut buffer).await?;
    let (message, _): (Message, usize) =
        bincode::serde::borrow_decode_from_slice(&buffer, bincode::config::standard())?;

    match message {
        Message::IdentityKey(m) => {
            server_id_key = m;
        }
        _ => panic!("Server didn't send ID key"),
    }

    stream_write.write(&Message::one_time_key(&account).bincode()?).await?;

    stream_read.read(&mut buffer).await?;
    let (message, _): (Message, usize) =
        bincode::serde::borrow_decode_from_slice(&buffer, bincode::config::standard())?;

    match message {
        Message::OneTimeKey(m) => {
            server_otk = m;
        }
        _ => panic!("Server didn't send OTK key"),
    }

    let mut session =
        account.create_outbound_session(SessionConfig::version_2(), server_id_key, server_otk);

    stream_write.write(&Message::message(&mut session, "Hello, server!").bincode()?).await?;

    stream_read.read(&mut buffer).await?;
    let (message, _): (Message, usize) =
        bincode::serde::borrow_decode_from_slice(&buffer, bincode::config::standard())?;

    match message {
        Message::Message(m) => match m {
            OlmMessage::PreKey(m) => {
                let result = account.create_inbound_session(server_id_key, &m)?;
                println!("{}", String::from_utf8_lossy(&result.plaintext))
            }
            _ => panic!("Server sent normal message"),
        },
        _ => panic!("Server didn't send message"),
    }

    // loop {
    //     tokio::select! {
    //         ready = stream_read.readable() => {
    //             match ready {
    //                 Ok(()) => match stream_read.try_read(&mut buffer) {
    //                     Ok(0) => {
    //                         println!("connection closed by peer");
    //                         break;
    //                     }
    //                     Ok(n) => {
    //                         println!("received {} bytes: {}", n, BASE64_STANDARD.encode(&buffer[..n]));

    //                         let (message, bytes): (Message, usize) = bincode::serde::borrow_decode_from_slice(&buffer[..n], bincode::config::standard())?;
    //                         match message {
    //                             Message::IdentityKey(m) => {},
    //                             Message::OneTimeKey(m) => {},
    //                             Message::Message(m) => {},
    //                         }

    //                     }
    //                     Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
    //                         continue;
    //                     }
    //                     Err(err) => {
    //                         eprintln!("failed to read from socket: {err}");
    //                         break;
    //                     }
    //                 },
    //                 Err(err) => {
    //                     eprintln!("socket readable() error: {err}");
    //                     break;
    //                 }
    //             }
    //         }
    //         stdin_res = stdin_reader.read_line(&mut stdin_line) => {
    //             match stdin_res {
    //                 Ok(0) => {
    //                     println!("stdin closed");
    //                     break;
    //                 }
    //                 Ok(_) => {
    //                     if let Err(err) = stream_write.write_all(stdin_line.as_bytes()).await {
    //                         eprintln!("failed to write to socket: {err}");
    //                         break;
    //                     }
    //                     stdin_line.clear();
    //                 }
    //                 Err(err) => {
    //                     eprintln!("failed to read stdin: {err}");
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 3 {
        if &args[1] == "--host" {
            handle_server(&args[2]).await?;
        } else if &args[1] == "--connect" {
            handle_client(&args[2]).await?;
        } else {
            println!("Missing <PORT>");
        }
    }

    return Ok(());

    // let port = 4443;
    // let listener = TcpListener::bind(("127.0.0.1", port)).await.expect("Failed to bind to port");

    // let alice = Account::new();
    // let mut bob = Account::new();

    // bob.generate_one_time_keys(1);
    // let bob_otk = *bob.one_time_keys().values().next().unwrap();

    // let mut alice_session = alice
    //     .create_outbound_session(SessionConfig::version_2(), bob.curve25519_key(), bob_otk);

    // bob.mark_keys_as_published();

    // let message = "Keep it between us, OK?";
    // let alice_msg = alice_session.encrypt(message);

    // if let OlmMessage::PreKey(m) = alice_msg.clone() {
    //     let result = bob.create_inbound_session(alice.curve25519_key(), &m)?;

    //     let mut bob_session = result.session;
    //     let what_bob_received = result.plaintext;

    //     assert_eq!(alice_session.session_id(), bob_session.session_id());

    //     assert_eq!(message.as_bytes(), what_bob_received);

    //     let bob_reply = "Yes. Take this, it's dangerous out there!";
    //     let bob_encrypted_reply = bob_session.encrypt(bob_reply).into();

    //     let what_alice_received = alice_session
    //         .decrypt(&bob_encrypted_reply)?;
    //     assert_eq!(what_alice_received, bob_reply.as_bytes());
    // }

    // let message = "Next message";
    // let alice_msg = alice_session.encrypt(message);

    // Ok(())
}
