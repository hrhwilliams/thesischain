use thesischain::peer::*;
use thesischain::routes::*;
use thesischain::startup::*;

async fn connect_peers(port1: u16, port2: u16) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    client
        .post(format!("http://localhost:{}/add_peer", port1))
        .json(&PeerMessage {
            address: format!("http://127.0.0.1:{}", port2),
        })
        .send()
        .await?;

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut handles = vec![];
    let mut ports = vec![];
    let mut apps = vec![];
    
    for _ in 0..4 {
        // spawn tasks that build and return the app instead of moving outer vectors into the task
        handles.push(tokio::spawn(async {
            KeyChain::build().await.expect("Failed to build app")
        }));
    }

    // wait for all builds to complete and collect the apps on this task
    let results = futures::future::join_all(handles).await;
    let mut handles = vec![];
    for res in results {
        let app = res.expect("task join failed");
        ports.push(app.port());
        apps.push(app);
    }

    for app in apps {
        handles.push(tokio::spawn(app.run()));
    }

    // randomly connect peers
    // for _ in 1..4 {
    //     let (i, j) = rand::thread_rng().r#gen::<(usize, usize)>();
    //     let p1 = ports[i % ports.len()];
    //     let p2 = ports[j % ports.len()];
    //     connect_peers(p1, p2).await?;
    // }

    // connect all peers to each other
    for i in 0..ports.len() {
        for j in 0..ports.len() {
            if i != j {
                let p1 = ports[i];
                let p2 = ports[j];
                connect_peers(p1, p2).await?;
                connect_peers(p2, p1).await?;
            }
        }
    }

    // print out url to access each peer
    for port in ports.iter() {
        println!("Peer running at http://127.0.0.1:{}", port);
    }

    futures::future::join_all(handles).await;
    Ok(())
}
