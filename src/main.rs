mod daemon{
    pub mod daemon;
    pub mod notif;
    pub mod log;
}
mod modules{
    pub mod monitoring;
}
use daemon::daemon::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::{io, u64};
use std::thread;
use std::time::Duration;

use crate::daemon::log::Log;
use crate::modules::monitoring;

const FILE_CONF:&str="/tmp/data.json";

#[tokio::main]
async fn main() -> io::Result<()> {
    run().await
}

async fn run() -> io::Result<()>{
    println!("Daemon started...");
   let _cpu_swap = tokio::spawn(async {
       loop{
           monitoring::monswap();
           monitoring::moncpu().await;
           monitoring::gpu().await;
           thread::sleep(Duration::from_secs(1));
       }
   });
   let _conf = tokio::spawn(async{
        loop {
            let path = PathBuf::from_str(FILE_CONF).expect("Error");
            let path2 = PathBuf::from_str("/tmp/d.json").expect("Error");
            let data = read_file(&path).expect("Error");
            let _ = Log::save_log("FILE_CONF", format!("data:{}",data));
            
            let mut file = File::create(&path2).expect("FILE");
            file.write_all(data.as_bytes()).expect("E");
            thread::sleep(Duration::from_secs(5));
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
