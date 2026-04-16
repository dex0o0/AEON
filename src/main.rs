#[macro_use]
mod macros;

mod daemon;
// mod daemon{
//     pub mod core;
//     pub mod notif;
//     pub mod log;
// }

mod modules{
    pub mod monitoring;
    pub mod backup;
}

use daemon::core::*;
use std::path::PathBuf;
use std::{env, io, u64};
use std::fs::{self, File};
use serde::{Serialize,Deserialize};
use std::time::Duration;
use crate::modules::monitoring;

// const FILE_CONF:&str="/tmp/data.json";
const FILE_DATA_PATH:&str=".config/AEON/config.json";

#[derive(Deserialize,Serialize,Debug)]
pub struct DataConf{
    cputsh:Option<f32>,
}

fn read_data(path:&PathBuf)-> Option<DataConf>{
    if !path.exists(){
        File::create(path).expect("Error:can't create config file");
    }
   let data = fs::read_to_string(path).expect("can't read data as config file");
   if let Ok(json) = serde_json::from_slice(data.as_bytes()){ // i don't known how write this 
       Some(json) // if you known please fixed 
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
    let conf = read_data(&path_conf).unwrap_or({
        DataConf{
            cputsh:Some(80.0),
        }
    });
    let cputsh = conf.cputsh.unwrap_or(80.0);
    let mut inter300mil = tokio::time::interval(Duration::from_millis(300));
    let mut inter60sec = tokio::time::interval(Duration::from_secs(60));
    let mut inter2sec = tokio::time::interval(Duration::from_secs(2));
    let _cpu_swap = tokio::spawn(async move{
        loop{
            inter300mil.tick().await;
            monitoring::monswap().await;
            monitoring::moncpu(cputsh).await;
            monitoring::check_mem().await;
           // thread::sleep(Duration::from_millis(300));
        }
    });
    let _disk=tokio::spawn(async move{
        loop {
            inter2sec.tick().await;
            monitoring::check_disk().await;
            // thread::sleep(Duration::from_secs(2));
        }
    });
   let _net_handle=tokio::spawn(async move{
       loop{
           inter60sec.tick().await;
            let _ = check_net().await;
            // thread::sleep(Duration::from_secs(60));
       }
   });
   let _ =tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
   Ok(())
}
