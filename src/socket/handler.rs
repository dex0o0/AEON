use std::sync::Arc;

use super::lib::respond;
use crate::modules::monitoring::{Icpu, Idisks, Systate};
use crate::modules::scan_sys::Sysinfo;
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;

#[derive(Serialize, Default, Debug, Deserialize)]
pub struct DiskInfoResponse {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gpuinfo {
    pub name: String,
    pub brand: String,
    pub memory_md: u64,
    pub temp_cel: u32,
    pub usage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cpuinfo {
    pub name: String,
    pub pcore: u32,
    pub tpros: u32,
    pub hypr: bool,
    pub temp: u32,
    pub usage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystateResponse {
    pub disk: Vec<DiskInfoResponse>,
    pub gpu: Gpuinfo,
}

pub async fn handler(
    stream: &mut UnixStream,
    msg: &str,
    state: Arc<tokio::sync::Mutex<Systate>>,
    idisk: Arc<tokio::sync::Mutex<Idisks>>,
    icpu: Arc<tokio::sync::Mutex<Icpu>>,
) {
    let cmd = msg.trim();
    match cmd {
        "PING" => respond(stream, "PING", "PONG").await,
        "STATUS" => {
            let disklist = trans_disk(idisk);
        }
        "CPUINFO" => {}
        "GPUINFO" => {}
        "DISKINFO" => {}
        _ => {
            let error_msg = format!("Error: Unknown command '{}'", cmd);
            respond(stream, &error_msg, "UNKNOWN").await;
        }
    }
}

async fn trans_disk(idisks: Arc<tokio::sync::Mutex<Idisks>>) -> Vec<DiskInfoResponse> {
    let idisks = idisks.lock().await;
    let mut vec = vec![];
    if let Ok(disks) = idisks.disk.lock() {
        disks.iter().for_each(|disk| {
            vec.push(DiskInfoResponse {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
            });
        });
    }
    vec
}
async fn transe_cpu() -> Cpuinfo {}
async fn transe_gpu() -> Gpuinfo {}
