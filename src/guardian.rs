use aeon::socket::lib::{socket_get, socket_send};
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::process::Command;
use tokio::time::{self, interval};

const SOCK_PATH: &str = "/tmp/AEON.sock";

async fn check_health() -> bool {
    match UnixStream::connect(SOCK_PATH).await {
        Ok(mut stream) => {
            if socket_send(&mut stream, "PING").await.is_err() {
                return false;
            }
            match socket_get(&mut stream).await {
                Ok(response) => response == "PONG",
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
async fn kill_frozen_damon() {
    let status = Command::new("pkill").args(["-9", "AEON"]).status().await;
    match status {
        Ok(s) if s.success() => println!("[GUARDIAN] AEON service die soon started"),
        _ => println!("[GUARDIAN] service not run soon started by systemd"),
    }
}

#[tokio::main]
async fn main() {
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        if !check_health().await {
            println!("AEON is down");
            kill_frozen_damon().await;
        }
    }
}
