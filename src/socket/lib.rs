use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    // process::Command,
    // thread,
    // time::Duration,
};

//this is function for Send message to unix socket
pub fn socket_send(mut stream: &UnixStream, msg: &str) -> std::io::Result<()> {
    let format_msg = format!("{}\n", msg);
    stream.write_all(format_msg.as_bytes())?;
    stream.flush()?;
    Ok(())
}

//this for Read message on unix socket
pub fn socket_get(stream: &UnixStream) -> std::io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut response = String::new();

    //read one line
    reader.read_line(&mut response)?;

    Ok(response.trim().to_string())
}

pub fn create_sock(path: &str) -> std::io::Result<UnixListener> {
    if fs::metadata(path).is_ok() {
        let _ = fs::remove_file(path);
    }
    UnixListener::bind(path)
}

pub fn respond(stream: &UnixStream, msg: &str, cmd_name: &str) {
    if let Err(e) = socket_send(stream, cmd_name) {
        log_error!("{}:{}:{}", msg, cmd_name, e);
    }
}
