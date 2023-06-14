use std::{thread, time::Duration};

use log::info;
use tinyp2p::{config::P2pConfig, Client, OutEvent, Topic};
use tokio::task;
use tokio_stream::{Stream, StreamExt};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut config = P2pConfig::default();
    if let Some(addr) = std::env::args().nth(1) {
        config.boot_node = addr.parse().ok();
    }

    let (client, event_stream, server) = tinyp2p::new(config).unwrap();

    // Run the p2p server
    task::spawn(server.run());

    // Periodically print the node status.
    let client_clone = client.clone();
    thread::spawn(move || get_node_status(client_clone));

    // Periodically send a request to one of the known peers.
    let client_clone = client.clone();
    thread::spawn(move || request(client_clone));

    // Periodically make a broadcast to the network.
    let client_clone = client.clone();
    thread::spawn(move || broadcast(client_clone));

    // Handle events from the p2p server.
    event_loop(event_stream, client).await;
}

fn get_node_status(client: Client) {
    let dur = Duration::from_secs(7);
    loop {
        thread::sleep(dur);
        let node_status = client.get_node_status();
        info!("📣 Node status: {:?}", node_status);
    }
}

fn request(client: Client) {
    let dur = Duration::from_secs(11);
    loop {
        thread::sleep(dur);
        let known_peers = client.get_known_peers();
        if known_peers.len() > 0 {
            let target = &known_peers[0];
            let request = "Hello, request!";
            info!("📣 >>>> Outbound request: {:?}", request);
            let response = client
                .blocking_request(target, request.as_bytes().to_vec())
                .unwrap();
            info!(
                "📣 <<<< Inbound response: {:?}",
                String::from_utf8_lossy(&response)
            );
        }
    }
}

fn broadcast(client: Client) {
    let dur = Duration::from_secs(13);
    loop {
        thread::sleep(dur);
        let topic = Topic::Block;
        let message = "Hello, a new block!";
        info!("📣 >>>> Outbound broadcast: {:?} {:?}", topic, message);
        let _ = client.broadcast(topic, message.as_bytes().to_vec());
    }
}

async fn event_loop(mut event_stream: impl Stream<Item = OutEvent> + Unpin, client: Client) {
    loop {
        match event_stream.next().await {
            Some(OutEvent::InboundRequest {
                request_id,
                payload,
            }) => {
                info!(
                    "📣 <<<< Inbound request: {:?}",
                    String::from_utf8_lossy(&payload)
                );
                let response = "Hello, response!";
                info!("📣 >>>> Outbound response: {:?}", response);

                client.send_response(request_id, Ok(response.as_bytes().to_vec()));
            }
            Some(OutEvent::Broadcast {
                source,
                topic,
                message,
            }) => {
                info!(
                    "📣 <<<< Inbound broadcast: {:?} {:?} {:?}",
                    source,
                    topic,
                    String::from_utf8_lossy(&message)
                );
            }
            None => continue,
        }
    }
}
