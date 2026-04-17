#[macro_use]
mod macros;

mod daemon{
    pub mod core;
    pub mod notif;
    pub mod log;
}

mod modules{
    pub mod monitoring;
    pub mod backup;
}

use daemon::core::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, io, u64};
use std::fs::{self, File};
use serde::{Serialize,Deserialize};
use std::time::Duration;
use crate::modules::monitoring::{self, Systate};

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
   match serde_json::from_str(&data) {
       Ok(json)=> Some(json),
       Err(e)=>{
            eprintln!("Error parsing data config:{}",e);
            None
       }
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
    
    let state = Arc::new(tokio::sync::Mutex::new(Systate::new()));

    let cputsh = conf.cputsh.ok_or_else(||{
        eprintln!("Error in read data cpu-treshold");
        80.0
    }).expect("Error to convet data cpu-treshold");

    let mut inter100mil = tokio::time::interval(Duration::from_millis(100));
    let mut inter60sec = tokio::time::interval(Duration::from_secs(1));
    let mut inter2sec = tokio::time::interval(Duration::from_secs(2));

    let state_clone = state.clone();
    let _cpu_swap = tokio::spawn(async move{
        loop{

            inter100mil.tick().await;
            let mut state = state_clone.lock().await;
            monitoring::monswap(&mut state.sys).await;
            monitoring::moncpu(&mut state,cputsh).await;
            monitoring::check_mem(&mut state.sys).await;
        }
    });

    let state_clone = state.clone();
    let _disk=tokio::spawn(async move{
        loop {
            inter2sec.tick().await;
            let state = state_clone.lock().await;
            monitoring::check_disk(&state.disk).await;
        }
    });

    let _net_handle=tokio::spawn(async move{
        loop{
            inter60sec.tick().await;
            let _ = check_net("8.8.8.8").await;
        }
    });

   let _ =tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
   Ok(())
}
