use packet_hacker::app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = std::env::var("PACKET_HACKER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()
        .expect("PACKET_HACKER_ADDR must be host:port");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind server listener");
    println!(
        "Packet Hacker listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app()).await.expect("serve app");
}
