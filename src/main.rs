#[macro_use]
mod macros;

mod daemon {
    pub mod core;
    pub mod log;
    pub mod notif;
}

mod modules {
    pub mod monitoring;
    pub mod rest;
}

use crate::modules::monitoring::{self, scan_processes, Icpu, Idisks, ProcessWatcher, Systate};
use crate::modules::rest::rest_run;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, io, u64};
use sysinfo::Process;

// const FILE_CONF:&str="/tmp/data.json";
const FILE_DATA_PATH: &str = ".config/AEON/config.json";

#[derive(Deserialize, Serialize, Debug)]
pub struct DataConf {
    cputsh: Option<f32>,
}

fn read_data(path: &PathBuf) -> Option<DataConf> {
    if !path.exists() {
        File::create(path).expect("Error:can't create config file");
    }
    let data = fs::read_to_string(path).expect("can't read data as config file");
    match serde_json::from_str(&data) {
        Ok(json) => Some(json),
        Err(e) => {
            eprintln!("Error parsing data config:{}", e);
            None
        }
    }
}

//MAIN function
#[tokio::main]
async fn main() -> io::Result<()> {
    run().await
}

async fn run() -> io::Result<()> {
    println!("Daemon started...");

    let homedir = env::home_dir().expect("Error");
    let path_conf = homedir.join(FILE_DATA_PATH);
    let conf = read_data(&path_conf).unwrap_or(DataConf { cputsh: Some(80.0) });

    let state = Arc::new(tokio::sync::Mutex::new(Systate::new()));
    let idisk = Arc::new(tokio::sync::Mutex::new(Idisks::new()));
    let icpu = Arc::new(tokio::sync::Mutex::new(Icpu::new()));

    let cputsh = conf
        .cputsh
        .ok_or_else(|| {
            eprintln!("Error in read data cpu-treshold");
            80.0
        })
        .expect("Error to convet data cpu-treshold");

    let mut inter100mil = tokio::time::interval(Duration::from_millis(100));
    let mut inter60sec = tokio::time::interval(Duration::from_secs(1));
    let mut inter2sec = tokio::time::interval(Duration::from_secs(2));

    // let state_for_serv = state.clone();
    // tokio::spawn(async move {
    //     let app = Router::new()
    //         .route("/health", get(heath_handle))
    //         .layer(CorsLayer::very_permissive())
    //         .with_state(state_for_serv);
    //     let lisener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    //     axum::serve(lisener,app).await.unwrap();
    // });

    tokio::spawn(rest_run());

    let state_clone = state.clone();
    let icpu_clone = icpu.clone();
    let _cpu_swap = tokio::spawn(async move {
        loop {
            inter100mil.tick().await;
            let mut state = state_clone.lock().await;
            let mut icpu = icpu_clone.lock().await;
            monitoring::monswap(&mut state).await;
            monitoring::moncpu(&mut state, &mut icpu, cputsh).await;
            monitoring::check_mem(&mut state).await;
        }
    });

    tokio::spawn(async {
        loop {
            // let _ = daemon::core::test(3000u32).await;
            let _ = tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = std::process::Command::new("clear").spawn();
        }
    });

    let idisks_clone = idisk.clone();
    let _disk = tokio::spawn(async move {
        loop {
            inter2sec.tick().await;
            let mut idisks = idisks_clone.lock().await;
            monitoring::check_disk(&mut idisks);
        }
    });

    let _net_handle = tokio::spawn(async move {
        loop {
            inter60sec.tick().await;
            let _ = monitoring::check_net("8.8.8.8").await;
        }
    });

    let proc_watcher = Arc::new(ProcessWatcher::new());
    let mut inter_proc = tokio::time::interval(Duration::from_secs(10));
    let state_for_proc = state.clone();
    let watcher_for_proc = proc_watcher.clone();
    tokio::spawn(async move {
        loop {
            inter_proc.tick().await;
            let mut state = state_for_proc.lock().await;
            scan_processes(&mut state, &watcher_for_proc);
        }
    });
    let _ = tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
    Ok(())
}

// async fn heath_handle(State(state):State<Arc<tokio::sync::Mutex<Systate>>>)-> String{
//     let state = state.lock().await;
//     let cpu = state.cpu_usage.lock().expect("e");
//     format!("CPU usage:{:.2}%",cpu)
// }
