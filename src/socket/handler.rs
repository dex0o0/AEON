use super::lib::respond;
use std::os::unix::net::UnixStream;

pub fn handler(stream: &UnixStream, msg: &str) {
    let cmd = msg.trim();
    match cmd {
        "PING" => respond(stream, "PING", "PONG"),
        _ => {
            let error_msg = format!("Error: Unknown command '{}'", cmd);
            respond(stream, &error_msg, "UNKNOWN");
        }
    }
}

