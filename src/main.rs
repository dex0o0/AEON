mod daemon{
    pub mod daemon;
    pub mod notif;
    pub mod log;
}
mod modules{
    pub mod monitoring;
}
use daemon::daemon::*;
use std::{io, u64};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    run().await
    //fork
    // match unsafe { libc::fork() } {
    //     -1 => return Err(io::Error::last_os_error()), // Fork error
    //     0 => { // Child process
    //         let _ =run().await;
    //         Ok(())
    //     }
    //     _ => { // Parent process
    //         // Parent exits immediately
    //         exit(0); 
    //     }
    // }
}

async fn run() -> io::Result<()>{
    println!("Daemon started...");
   let _swap_handle = tokio::spawn(async {
       loop{
           check_swap();
           thread::sleep(Duration::from_secs(30));
       }
   }); 
   let _cpu_handle = tokio::spawn(async {
       loop{
           check_cpu().await;
           thread::sleep(Duration::from_secs(1));
       }
   });
   let _net_handle=tokio::spawn(async {
       loop{
            check_net().await;
            thread::sleep(Duration::from_secs(60));
       }
   });
   let _ =tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
   Ok(())
}
