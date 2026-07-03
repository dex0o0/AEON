use super::lib::respond;
use tokio::net::UnixStream;

pub async fn handler(stream: &mut UnixStream, msg: &str) {
    let cmd = msg.trim();
    match cmd {
        "PING" => respond(stream, "PING", "PONG").await,
        _ => {
            let error_msg = format!("Error: Unknown command '{}'", cmd);
            respond(stream, &error_msg, "UNKNOWN").await;
        }
    }
}
