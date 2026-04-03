mod daemon{
    pub mod notif;
    pub mod log;
}
use std::io;
use std::process::exit;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    //fork
    match unsafe { libc::fork() } {
        -1 => return Err(io::Error::last_os_error()), // Fork error
        0 => { // Child process
            let _ =run().await;
            Ok(())
        }
        _ => { // Parent process
            // Parent exits immediately
            exit(0); 
        }
    }
}

async fn run() -> io::Result<()> {
    println!("Daemon started...");
    loop {


        
        thread::sleep(Duration::from_secs(10));
    }
}
