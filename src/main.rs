mod daemon{
    pub mod daemon;
    pub mod notif;
    pub mod log;
}
mod modules{
    pub mod monitoring;
    pub mod backup;
}
use daemon::daemon::*;
use std::path::PathBuf;
use std::{env, io, u64};
use std::thread;
use std::fs;
use serde::{Serialize,Deserialize};
use std::time::Duration;
use crate::modules::monitoring;

const FILE_CONF:&str="/tmp/data.json";
const FILE_DATA_PATH:&str=".config/AEON/config.json";

#[derive(Deserialize,Serialize,Debug)]
pub struct DataConf{
    cputsh:Option<f32>,
}

fn read_data(path:&PathBuf)-> Option<DataConf>{
   let data = fs::read_to_string(path).expect("ERRORS");
   if let Ok(json) = serde_json::from_slice(data.as_bytes()){
       Some(json)
   }else {
       None
   }
   
}

#[tokio::main]  
async fn main() -> io::Result<()> {
    run().await
}

async fn run() -> io::Result<()>{
    println!("Daemon started...");
    let homedir = env::home_dir().expect("Error");
    let path_conf = homedir.join(FILE_DATA_PATH);
    let conf = read_data(&path_conf).expect("Error");
    let cputsh = conf.cputsh.unwrap_or(80.0);
    let _cpu_swap = tokio::spawn(async move{
        loop{
           monitoring::monswap();
           monitoring::moncpu(cputsh).await;
           monitoring::gpu().await;
           thread::sleep(Duration::from_secs(1));
        }
    });
    let _disk=tokio::spawn(async {
        loop {
            monitoring::check_disk().await;
            thread::sleep(Duration::from_secs(2));
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
