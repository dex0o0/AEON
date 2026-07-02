use aeon::socket::lib::{socket_get, socket_send};
use std::thread;
use std::time::Duration;
use std::{os::unix::net::UnixStream, process::Command};

const SOCK_PATH: &str = "/tmp/AEON.sock";

fn check_health() -> bool {
    match UnixStream::connect(SOCK_PATH) {
        Ok(stream) => {
            if socket_send(&stream, "PING").is_err() {
                return false;
            }
            match socket_get(&stream) {
                Ok(response) => response == "PONG",
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
fn kill_frozen_damon() {
    let status = Command::new("pkill").args(["-9", "AEON"]).status();
    match status {
        Ok(s) if s.success() => println!("[GUARDIAN] AEON service die soon started"),
        _ => println!("[GUARDIAN] service not run soon started by systemd"),
    }
}

fn main() {
    loop {
        if !check_health() {
            println!("AEON is down");
            kill_frozen_damon();
        } else {
            println!("OK");
        }
        println!("run");
        thread::sleep(Duration::from_secs(10));
    }
}
