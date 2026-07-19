use std::fs;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

//this is function for Send message to unix socket
pub async fn socket_send(stream: &mut UnixStream, msg: &str) -> std::io::Result<()> {
    let format_msg = format!("{}\n", msg);
    stream.write_all(format_msg.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

//this for Read message on unix socket
pub async fn socket_get(stream: &mut UnixStream) -> std::io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut response = String::new();

    //read one line
    reader.read_line(&mut response).await?;

    Ok(response.trim().to_string())
}

pub fn create_sock(path: &str) -> std::io::Result<UnixListener> {
    if fs::metadata(path).is_ok() {
        let _ = fs::remove_file(path);
    }
    UnixListener::bind(path)
}

pub async fn respond(stream: &mut UnixStream, msg: &str, cmd_name: &str) {
    if let Err(e) = socket_send(stream, cmd_name).await {
        log_error!("{}:{}:{}", msg, cmd_name, e);
    }
}
