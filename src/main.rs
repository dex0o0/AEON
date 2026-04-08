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
}

async fn run() -> io::Result<()>{
    println!("Daemon started...");
   let _cpu_swap = tokio::spawn(async {
       loop{
           check_swap();
           check_cpu().await;
           thread::sleep(Duration::from_secs(1));
       }
   });
   let _net_handle=tokio::spawn(async {
       loop{
            let _ = check_net().await;
            thread::sleep(Duration::from_secs(60));
       }
   });
   let _ =tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
   Ok(())
}
