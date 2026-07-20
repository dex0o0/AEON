use std::{
    collections::HashMap,
    io,
    process::Command,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use sysinfo::{Disks, System};

static NETWORK_IS_UP: AtomicBool = AtomicBool::new(false);
static MEM_WARNING_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct Idisks {
    pub disk: Mutex<Disks>,
    pub disk_warining_active: AtomicBool,
    pub disks_fill: Vec<String>,
}

#[derive(Debug)]
pub struct Icpu {
    pub cpu_usage: Mutex<f32>,
    pub cpu_temp: Mutex<u32>,
    pub cpu_warning_active: AtomicBool,
    pub cpu_warning_start: Mutex<Option<Instant>>,
    pub cpu_100_notif: AtomicBool,
    pub cpu_100_start: Mutex<Option<Instant>>,
}

#[derive(Debug)]
pub struct Systate {
    pub sys: System,
    pub mem_useag: Mutex<f32>,
    pub swap_usage: Mutex<f32>,
}

#[derive(Debug)]
struct IsOverheadCpu {
    value: f32,
    status: bool,
}

impl Default for Icpu {
    fn default() -> Self {
        Self {
            cpu_usage: Mutex::new(0.0),
            cpu_temp: Mutex::new(0),
            cpu_warning_active: AtomicBool::new(false),
            cpu_warning_start: Mutex::new(None),
            cpu_100_notif: AtomicBool::new(false),
            cpu_100_start: Mutex::new(None),
        }
    }
}

impl Default for Systate {
    fn default() -> Self {
        Self {
            sys: System::new(),
            mem_useag: Mutex::new(0.0),
            swap_usage: Mutex::new(0.0),
        }
    }
}

impl Default for Idisks {
    fn default() -> Self {
        Self {
            disk: Mutex::new(Disks::new_with_refreshed_list()),
            disk_warining_active: AtomicBool::new(false),
            disks_fill: Vec::new(),
        }
    }
}

//check SWAP usage
pub fn monswap(state: &mut Systate) {
    state.sys.refresh_memory();
    let swap = sysinfo::System::free_swap(&state.sys);
    let total = sysinfo::System::total_swap(&state.sys);

    let mut mem = state.swap_usage.lock().unwrap();
    *mem = (total - swap) as f32 / 1024.0 / 1024.0;
    if total == 0 {
        return;
    }
    if (total - swap) as f32 >= total as f32 * 0.8 {
        let massage = "your use of partition swap is high\nplease check".to_string();
        notif_log_sys!(massage);
    }
}

//check CPU usage
pub fn moncpu(state: &mut Systate, icpu: &mut Icpu, value: f32) {
    state.sys.refresh_cpu_usage();
    let cpu_usage = state.sys.global_cpu_usage();
    let mut cpu = icpu.cpu_usage.lock().expect("E");
    *cpu = cpu_usage;
    drop(cpu);

    //100% send now notify
    if cpu_usage >= 99.0 {
        let mut start_opt = icpu
            .cpu_100_start
            .lock()
            .expect("Error can't lock cpu_100_start");
        if start_opt.is_none() {
            *start_opt = Some(Instant::now());
        } else {
            let elipsed = start_opt.expect("Error to ger elipsed").elapsed();
            if elipsed >= Duration::from_secs(3) {
                let already_notifed = icpu.cpu_100_notif.load(Ordering::SeqCst);

                if !already_notifed {
                    let massage = format!("oh your CPU max usage:{:.2}%", cpu_usage);
                    notif_log_sys!(massage);
                    icpu.cpu_100_notif.store(true, Ordering::SeqCst);
                }
            }
        }
    }
    //if CPU usage for 5sec > value notify warning
    if cpu_usage > value {
        let mut start_opt = icpu
            .cpu_warning_start
            .lock()
            .expect("Error can't lock cpu_warning_start lock");

        if start_opt.is_none() {
            *start_opt = Some(Instant::now());
        } else {
            let elapsed = start_opt.expect("Error get elapsed").elapsed();
            if elapsed >= Duration::from_secs(10) {
                let already_warned = icpu.cpu_warning_active.load(Ordering::SeqCst);
                if !already_warned {
                    let massage = format!(
                        "your CPU for 5 secend is high\n=>{}%",
                        state.sys.global_cpu_usage()
                    );
                    notif_log_sys!(massage);
                    icpu.cpu_warning_active.store(true, Ordering::SeqCst);
                }
            }
        }
    } else {
        let mut start_opt = icpu
            .cpu_warning_start
            .lock()
            .expect("Error to unlock cpu_warning_start");

        *start_opt = None;
        if icpu.cpu_warning_active.load(Ordering::SeqCst) {
            icpu.cpu_warning_active.store(false, Ordering::SeqCst);
        }
    }

    let comp = sysinfo::Components::new_with_refreshed_list();
    let temp = comp
        .iter()
        .find(|c| {
            c.label().to_lowercase().contains("cpu") || c.label().to_lowercase().contains("core")
        })
        .map(|c| c.temperature().unwrap_or(0.0) as u32)
        .unwrap_or(0);

    if let Ok(mut cpu_temp_lock) = icpu.cpu_temp.lock() {
        *cpu_temp_lock = temp;
    }
}

//check DISK usage
pub fn check_disk(state: &mut Idisks) {
    // let disks = Disks::new_with_refreshed_list();

    state.disk.lock().unwrap().iter().for_each(|disk| {
        let total = disk.total_space();
        let free_space = disk.available_space();
        let use_space = total - free_space;
        let zone90 = total as f32 * 0.9;
        let montpoint = disk.mount_point().display();

        if use_space as f32 >= zone90 {
            let masssage = format!(
                "storage space filling\n\
                disk\ttotal\tusage\tfree\n\
                {}\t{:.2}G\t{:.2}G\t{:.2}G\t{}",
                disk.name().to_string_lossy(),
                (total as f32 / 1024.0 / 1024.0 / 1024.0),
                (use_space as f32 / 1024.0 / 1024.0 / 1024.0),
                (free_space as f32 / 1024.0 / 1024.0 / 1024.0),
                montpoint
            );

            let body = format!(
                "disk:{},is filling please check",
                disk.name().to_string_lossy()
            );
            if !state.disks_fill.contains(&body) {
                state.disks_fill.push(body);
            }
            log_sys!("{}", masssage);
        } else {
            // state.disk_warining_active.store(false, Ordering::SeqCst);
        }
    });

    if !state.disks_fill.is_empty() {
        if !state.disk_warining_active.load(Ordering::SeqCst) {
            state.disks_fill.iter().for_each(|fdisk| {
                notif_send!("{}", fdisk);
            });
            state.disk_warining_active.store(true, Ordering::SeqCst);
        }
    } else {
        state.disk_warining_active.store(false, Ordering::SeqCst);
    }
}

//check MEMORY usage
pub fn check_mem(state: &mut Systate) {
    state.sys.refresh_memory();
    let total = state.sys.total_memory();
    let usage = state.sys.used_memory();

    let mut mem = state.mem_useag.lock().unwrap();
    *mem = usage as f32 / 1024.0 / 1024.0 / 1024.0;
    drop(mem);

    if usage as f32 >= (total as f32 * 0.8) {
        if !MEM_WARNING_ACTIVE.load(Ordering::SeqCst) {
            let massage = format!(
                "mempry usage is very high:{}",
                (usage as f32 / 1024.0 / 1024.0 / 1024.0)
            );
            notif_log_sys!(massage);
            MEM_WARNING_ACTIVE.store(true, Ordering::SeqCst);
        }
    } else {
        MEM_WARNING_ACTIVE.store(false, Ordering::SeqCst);
    }
}

/// <center><h1>Ping function</h1></center>
/// <hr/>
/// <br/>
/// <h5>Ping with</h5>
/// <br/>
/// unix command "ping 8.8.8.8"
///
/// you can use this function to other file
///```
/// check_net("<ip address>").await;
///```
/// <h2>for example</h2>
///
/// ```
/// check_net("8.8.8.8").await;
/// ```
#[cfg(unix)]
pub async fn check_net(ip: &str) -> io::Result<()> {
    let ping = Command::new("ping").args(["-W 5", "-c 1", ip]).output();
    match ping {
        Ok(output) => {
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);

            if output.status.success() {
                //net up
                //check status
                if !was_net_up {
                    notif_send!("network is up");
                    log_error!("network is up");
                    NETWORK_IS_UP.store(true, Ordering::SeqCst);
                }
            } else {
                //net down
                //check status
                if was_net_up {
                    notif_send!("network is down");
                    log_error!("network is down");
                    NETWORK_IS_UP.store(false, Ordering::SeqCst);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to execute ping command:{}", e);
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);
            if was_net_up {
                let massage = format!("network connection lost:{}", e);
                notif_send!("{}", massage);
                log_error!("network connection lost");
                NETWORK_IS_UP.store(false, Ordering::SeqCst);
            }
            return Err(e);
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub mem_usage: f64,
    pub run_time: u64,
    pub first_seen: Instant,
}

#[derive(Debug)]
pub struct ProcessWatcher {
    pub tracked_processes: Mutex<HashMap<u32, Process>>,
    pub cpu_threshold: f32,
    pub mem_threshold: f64,
    pub time_threshold: u64,
    pub warning_active: AtomicBool,
    pub is_first_run: AtomicBool,
}

impl Default for ProcessWatcher {
    fn default() -> Self {
        Self {
            tracked_processes: Mutex::new(HashMap::new()),
            cpu_threshold: 80.0,
            mem_threshold: 1024.0 * 1024.0 * 5.0,
            time_threshold: 300,
            warning_active: AtomicBool::new(false),
            is_first_run: AtomicBool::new(true),
        }
    }
}

pub fn scan_processes(state: &mut Systate, watcher: &ProcessWatcher) {
    let is_first = watcher.is_first_run.swap(false, Ordering::SeqCst);

    state.sys.refresh_cpu_usage();

    state
        .sys
        .refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    if is_first {
        return;
    }

    let processes = state.sys.processes();
    let mut tracked = watcher.tracked_processes.lock().unwrap();
    let now = Instant::now();

    tracked.retain(|pid, _| processes.keys().any(|p| p.as_u32() == *pid));
    tracked.shrink_to_fit();
    let num_cores = state.sys.cpus().len() as f32;

    processes.iter().for_each(|(pid, process)| {
        let raw_cpu = process.cpu_usage();
        let cpu = is_overhead_cpu(raw_cpu, num_cores);
        let mem = process.memory() as f64 / 1024.0;

        // dbg!(mem, watcher.mem_threshold);
        // dbg!(raw_cpu, num_cores, cpu);

        let name = process.name().to_string_lossy().to_string();

        let is_suspicious = cpu.status || mem > watcher.mem_threshold;

        // dbg!(
        //     mem,
        //     watcher.mem_threshold,
        //     mem,
        //     watcher.mem_threshold,
        //     raw_cpu
        // );

        if is_suspicious {
            if let Some(existing) = tracked.get_mut(&pid.as_u32()) {
                existing.cpu_usage = cpu.value;
                existing.mem_usage = mem;

                existing.run_time = now.duration_since(existing.first_seen).as_secs();
                if existing.run_time >= watcher.time_threshold {
                    let msg = format!(
                        "Suspicious process detected\n\
                        Name:{}\n\
                        PID:{}\n\
                        CPU:{:.2}%\n\
                        Memory:{:.2}MB",
                        name,
                        pid,
                        cpu.value,
                        mem / 1024.0
                    );
                    // notif_log_sys!(msg);
                    log_sys!("{}", msg);
                }
            } else {
                tracked.insert(
                    pid.as_u32(),
                    Process {
                        pid: pid.as_u32(),
                        name: name.clone(),
                        cpu_usage: cpu.value,
                        mem_usage: mem,
                        run_time: 0,
                        first_seen: now,
                    },
                );
            }
        } else {
            if tracked.remove(&pid.as_u32()).is_some() {
                let msg = format!(" {} PID {} is now normal", name, pid);
                log_sys!("Process:{}", msg);
                // notif_log_sys!(msg);
                // dbg!(msg);
            }
        }
    });
    if tracked.is_empty() {
        tracked.shrink_to_fit();
    }
}

fn is_overhead_cpu(raw_core: f32, num_core: f32) -> IsOverheadCpu {
    if raw_core == 0.0 {
        return IsOverheadCpu {
            value: raw_core,
            status: false,
        };
    }

    let total_cpu_usage = raw_core / num_core;
    if total_cpu_usage > 80.0 {
        IsOverheadCpu {
            value: total_cpu_usage,
            status: true,
        }
    } else {
        IsOverheadCpu {
            value: total_cpu_usage,
            status: false,
        }
    }
}
