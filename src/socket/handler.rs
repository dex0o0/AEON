use super::lib::respond;
use crate::modules::monitoring::{Icpu, Idisks};
use nvml_wrapper::Nvml;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::{fs, path::Path, sync::Arc};
use tokio::net::UnixStream;

static NVML_INSTANCE: OnceLock<Option<Nvml>> = OnceLock::new();

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
    pub cpu: Cpuinfo,
    pub gpu: Gpuinfo,
}

#[derive(Clone, Debug)]
pub struct StaticSystemInfo {
    pub cpu_brand: String,
    pub physical_cores: u32,
    pub total_processor: u32,
    pub hyperthreading_enabled: bool,
    pub gpu: String,
    pub gpu_brand: String,
    pub gpu_memory_as_mb: u64,
    pub gpu_temperatuer_celsius: u32,
}

pub async fn handler(
    stream: &mut UnixStream,
    msg: &str,
    idisk: Arc<RwLock<Idisks>>,
    icpu: Arc<RwLock<Icpu>>,
    static_info: Arc<StaticSystemInfo>,
) {
    let cmd = msg.trim();

    match cmd {
        "PING" => respond(stream, "PING", "PONG").await,
        "STATUS" => {
            let disklist = transe_disk(idisk).await;
            let cpuinfo = transe_cpu(icpu, &static_info).await;
            let gpuinfo = transe_gpu(&static_info).await;

            let response_data = serde_json::json!( {
                "disk": disklist,
                "cpu_usage": cpuinfo.usage,
                "cpu_temp":cpuinfo.temp,
                "gpu": gpuinfo,
            });

            if let Ok(json_data) = serde_json::to_string(&response_data) {
                respond(stream, "STATUS", &json_data).await;
            }
        }
        "CPUINFO" => {
            let cpu = transe_cpu(icpu, &static_info).await;
            if let Ok(json_data) = serde_json::to_string(&cpu) {
                respond(stream, "CPUINFO", &json_data).await;
            }
        }
        "GPUINFO" => {
            let gpu = transe_gpu(&static_info).await;
            if let Ok(json_data) = serde_json::to_string(&gpu) {
                respond(stream, "GPUINFO", &json_data).await;
            }
        }
        "DISKINFO" => {
            let disk = transe_disk(idisk).await;
            if let Ok(json_data) = serde_json::to_string(&disk) {
                respond(stream, "DISKINFO", &json_data).await;
            }
        }
        _ => {
            let error_msg = format!("Error: Unknown command '{}'", cmd);
            respond(stream, &error_msg, "UNKNOWN").await;
        }
    }
}

async fn transe_disk(idisks: Arc<RwLock<Idisks>>) -> Vec<DiskInfoResponse> {
    let idisks = idisks.read();
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
async fn transe_cpu(icpu: Arc<RwLock<Icpu>>, static_info: &StaticSystemInfo) -> Cpuinfo {
    let icpug = icpu.read();
    let current_usage = icpug.cpu_usage.lock().map(|u| *u).unwrap_or(0.0);
    let current_temp = icpug.cpu_temp.lock().map(|t| *t).unwrap_or(0);

    Cpuinfo {
        name: static_info.cpu_brand.clone(),
        pcore: static_info.physical_cores,
        tpros: static_info.total_processor,
        hypr: static_info.hyperthreading_enabled,
        temp: current_temp,
        usage: current_usage,
    }
}

async fn transe_gpu(static_info: &StaticSystemInfo) -> Gpuinfo {
    let brand = static_info.gpu.to_lowercase();
    let gpu_usage = if brand.contains("nvidia") {
        get_nvidia_gpu_usage().unwrap_or(0.0)
    } else if brand.contains("amd") || brand.contains("radeon") {
        get_amd_gpu_usage().unwrap_or(0.0)
    } else {
        0.0
    };

    Gpuinfo {
        name: static_info.gpu.clone(),
        brand: static_info.gpu_brand.clone(),
        memory_md: static_info.gpu_memory_as_mb,
        temp_cel: static_info.gpu_temperatuer_celsius,
        usage: gpu_usage,
    }
}

fn get_amd_gpu_usage() -> Option<f32> {
    let paths = [
        "/sys/class/drm/card0/device/gpu_busy_percent",
        "/sys/class/drm/card1/device/gpu_busy_percent",
    ];
    for path in paths {
        if Path::new(&path).exists()
            && let Ok(val) = fs::read_to_string(path)
            && let Ok(u) = val.trim().parse::<f32>()
        {
            return Some(u);
        }
    }
    None
}

fn get_nvidia_gpu_usage() -> Option<f32> {
    let nvml = NVML_INSTANCE.get_or_init(|| Nvml::init().ok());
    let nvml_ref = nvml.as_ref()?;
    let device = nvml_ref.device_by_index(0).ok()?;
    let utilization = device.utilization_rates().ok()?;
    Some(utilization.gpu as f32)
}
