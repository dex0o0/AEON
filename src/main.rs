#[macro_use]
mod macros;

use aeon::modules::monitoring::{self, scan_processes, Icpu, Idisks, ProcessWatcher, Systate};
use mimalloc::MiMalloc;
mod cli;
use aeon::modules::scan_sys::Sysinfo;
use aeon::socket::{
    handler,
    lib::{create_sock, socket_get},
};
use cli::DataConf;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, io};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// const FILE_CONF:&str="/tmp/data.json";
const FILE_DATA_PATH: &str = ".config/AEON/config.json";
const SOCK_PATH: &str = "/tmp/AEON.sock";

async fn read_data(path: &PathBuf) -> io::Result<DataConf> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, b"{}").await?;
    }
    let data = tokio::fs::read_to_string(path).await?;
    let json: DataConf = serde_json::from_str(&data).unwrap_or(DataConf { cputsh: Some(80.0) });
    Ok(json)
}

//MAIN function
#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    run().await
}

async fn run() -> io::Result<()> {
    println!("Daemon started...");

    let homedir = env::home_dir().expect("Error");
    let path_conf = homedir.join(FILE_DATA_PATH);
    let conf = read_data(&path_conf)
        .await
        .unwrap_or(DataConf { cputsh: Some(80.0) });

    let state = Arc::new(RwLock::new(Systate::default()));
    let idisk = Arc::new(RwLock::new(Idisks::default()));
    let icpu = Arc::new(RwLock::new(Icpu::default()));

    let static_info = {
        let mut sys_static = Sysinfo::default();
        let _ = sys_static.auto_fill();

        Arc::new(aeon::socket::handler::StaticSystemInfo {
            cpu_brand: sys_static.cpu_brand,
            physical_cores: sys_static.physical_cores,
            total_processor: sys_static.total_processor,
            hyperthreading_enabled: sys_static.hyperthreading_enabled,
            gpu: sys_static.gpu,
            gpu_brand: sys_static.gpu_brand,
            gpu_memory_as_mb: sys_static.gpu_memory_as_mb,
            gpu_temperatuer_celsius: sys_static.gpu_temperatuer_celsius,
        })
    };

    let _ = start_sock_serv(SOCK_PATH, idisk.clone(), icpu.clone(), static_info.clone()).await;

    let cputsh = conf
        .cputsh
        .ok_or_else(|| {
            eprintln!("Error in read data cpu-treshold");
            80.0
        })
        .expect("Error to convet data cpu-treshold");

    // let mut inter100mil = tokio::time::interval(Duration::from_millis(100));

    let mut inter1sec = tokio::time::interval(Duration::from_secs(1));
    let mut inter60sec = tokio::time::interval(Duration::from_secs(60));
    let mut inter2sec = tokio::time::interval(Duration::from_secs(2));

    let state_clone = Arc::clone(&state);
    let icpu_clone = Arc::clone(&icpu);
    tokio::spawn(async move {
        loop {
            inter1sec.tick().await;
            {
                let mut state = state_clone.write();
                let mut icpu = icpu_clone.write();

                monitoring::monswap(&mut state);
                monitoring::moncpu(&mut state, &mut icpu, cputsh);
                monitoring::check_mem(&mut state);
            }
        }
    });

    let idisks_clone = Arc::clone(&idisk);
    tokio::spawn(async move {
        loop {
            inter2sec.tick().await;
            {
                let mut idisks = idisks_clone.write();
                monitoring::check_disk(&mut idisks);
            }
        }
    });

    tokio::spawn(async move {
        loop {
            inter60sec.tick().await;
            {
                let _ = monitoring::check_net("8.8.8.8").await;
            }
        }
    });

    let proc_watcher = Arc::new(ProcessWatcher::default());
    let mut inter_proc = tokio::time::interval(Duration::from_secs(10));
    let state_for_proc = Arc::clone(&state);
    let watcher_for_proc = Arc::clone(&proc_watcher);

    tokio::spawn(async move {
        loop {
            inter_proc.tick().await;
            {
                let mut state = state_for_proc.write();
                scan_processes(&mut state, &watcher_for_proc);
            }
        }
    });
    let _ = tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
    Ok(())
}

async fn start_sock_serv(
    sock_path: &str,
    idisk: Arc<RwLock<Idisks>>,
    icpu: Arc<RwLock<Icpu>>,
    static_info: Arc<aeon::socket::handler::StaticSystemInfo>,
) -> io::Result<()> {
    let listener = create_sock(sock_path)?;
    println!("socket run on path: '{}'", sock_path);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, _addr)) => {
                    let idisk_clone = Arc::clone(&idisk);
                    let icpu_clone = Arc::clone(&icpu);
                    let static_info = Arc::clone(&static_info);

                    tokio::spawn(async move {
                        match socket_get(&mut stream).await {
                            Ok(msg) => {
                                handler::handler(
                                    &mut stream,
                                    &msg,
                                    idisk_clone,
                                    icpu_clone,
                                    static_info,
                                )
                                .await;
                            }
                            Err(e) => eprintln!("Error to read message:{}", e),
                        }
                    });
                }
                Err(e) => eprintln!("Error:{e}"),
            }
        }
    });
    Ok(())
}
