use std::{io,process::Command, sync::{Mutex, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use sysinfo::{Disks, System};

static NETWORK_IS_UP:AtomicBool=AtomicBool::new(false);
static DISK_WARNING_ACTIVE:AtomicBool=AtomicBool::new(false);
static MEM_WARNING_ACTIVE:AtomicBool=AtomicBool::new(false);

#[derive(Debug)]
pub struct Systate{
    pub sys:System,
    pub disk:Mutex<Disks>,
    pub cpu_usage:Mutex<f32>,
    pub mem_useag:Mutex<f32>,
    pub swap_usage:Mutex<f32>,
    pub cpu_warning_active:AtomicBool,
    pub cpu_warning_start:Mutex<Option<Instant>>,
    pub cpu_100_notif:AtomicBool,
}

impl Systate {
    pub fn new()->Self{
        Self{
            sys:System::new_all(),
            disk:Mutex::new(Disks::new_with_refreshed_list()),
            cpu_usage:Mutex::new(0.0),
            mem_useag:Mutex::new(0.0),
            swap_usage:Mutex::new(0.0),
            cpu_warning_active:AtomicBool::new(false),
            cpu_warning_start:Mutex::new(None),
            cpu_100_notif:AtomicBool::new(false),
        }
    }
}

//check SWAP usage
pub async fn monswap(state:&mut Systate){
    state.sys.refresh_memory();
    let swap = sysinfo::System::free_swap(&state.sys);
    let total = sysinfo::System::total_swap(&state.sys);
    
    let mut mem = state.swap_usage.lock().unwrap();
    *mem = (total - swap ) as f32 /1024.0/1024.0;
    if total == 0{
        return;
    }
    if (total - swap) as f32 >= total as f32 * 0.8 {
        let massage = "your use of partition swap is high\nplease check".to_string();
        notif_log_sys!(massage);
    }
} 
//check CPU usage
pub async fn moncpu(state:&mut Systate , value:f32){
    state.sys.refresh_cpu_usage();
    let cpu_usage = state.sys.global_cpu_usage();
    let mut cpu = state.cpu_usage.lock().expect("E");
    *cpu = cpu_usage;
    drop(cpu);
    //100% send now notify
    if cpu_usage >= 99.0{
        let already_notifed = state.cpu_100_notif.load(Ordering::SeqCst);
        if !already_notifed{
            let massage = format!("oh your CPU max usage:{:.2}%",cpu_usage);
            notif_log_sys!(massage);
            state.cpu_100_notif.store(true, Ordering::SeqCst);
        }
    }else {
        state.cpu_100_notif.store(false, Ordering::SeqCst);
    }
    
    //if CPU usage for 5sec > value notify warning
    if cpu_usage > value{
        let mut start_opt = state.cpu_warning_start.lock().expect("Error can't lock cpu_warning_start lock");

        if start_opt.is_none(){
            *start_opt = Some(Instant::now());
        }else {
            let elapsed = start_opt.expect("Error get elapsed").elapsed();
            if elapsed >= Duration::from_secs(5){
                let already_warned = state.cpu_warning_active.load(Ordering::SeqCst);
                if !already_warned{
                    let massage = format!("your CPU for 5 secend is high\n=>{}%",state.sys.global_cpu_usage());
                    notif_log_sys!(massage);
                    state.cpu_warning_active.store(true,Ordering::SeqCst);
                }
            }
        }

    }else {

        let mut start_opt = state.cpu_warning_start.lock().expect("Error to unlock cpu_warning_start");

        *start_opt = None;
        if state.cpu_warning_active.load(Ordering::SeqCst){
            state.cpu_warning_active.store(false, Ordering::SeqCst);
        }
    }
}



//check DISK usage 
pub fn check_disk(disks:&Disks){
    // let disks = Disks::new_with_refreshed_list();
    disks.iter().for_each(|disk| {

        let total = disk.total_space();
        let free_space = disk.available_space();
        let use_space = total - free_space;
        let zone90 = total as f32 * 0.9;
        let montpoint = disk.mount_point().display();

        if use_space as f32 >= zone90{
            if !DISK_WARNING_ACTIVE.load(Ordering::SeqCst){
                let masssage = format!("storage space filling\n\
                    disk\ttotal\tusage\tfree\n\
                    {}\t{:.2}G\t{:.2}G\t{:.2}G\t{}",
                    disk.name().to_string_lossy(),
                    (total as f32 / 1024.0/1024.0/1024.0),
                    (use_space as f32 /1024.0/1024.0/1024.0),
                    (free_space as f32 /1024.0/1024.0/1024.0),
                    montpoint);

                log_sys!("{}",masssage);
                notif_send!("{}",format!("disk:{},is filling please check",disk.name().to_string_lossy()));
                DISK_WARNING_ACTIVE.store(true, Ordering::SeqCst);
            }
        }else {
            DISK_WARNING_ACTIVE.store(false, Ordering::SeqCst);
        }

    }); 
}

//check MEMORY usage
pub async fn check_mem(state:&mut Systate){
    state.sys.refresh_memory();
    let total = state.sys.total_memory();
    let usage = state.sys.used_memory();
    

    let mut mem = state.mem_useag.lock().unwrap();
    *mem = usage as f32/1024.0/1024.0/1024.0;
    drop(mem);


    if usage as f32 >= (total as f32 * 0.8){
        if !MEM_WARNING_ACTIVE.load(Ordering::SeqCst){
            let massage = format!("mempry usage is very high:{}",(usage as f32/1024.0/1024.0/1024.0));
            notif_log_sys!(massage);
            MEM_WARNING_ACTIVE.store(true, Ordering::SeqCst);
        }
    }else {
        MEM_WARNING_ACTIVE.store(false, Ordering::SeqCst);
    }
}


pub async fn check_net(ip:&str)-> io::Result<()>{
    let ping = Command::new("ping")
        .args(["-W 5","-c 1",ip])
        .output();
    match ping {
        Ok(output)=>{
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);
            
            if output.status.success(){
                //net up 
                //check status
                if !was_net_up{
                    notif_send!("network is up");
                    log_error!("network is up");
                    NETWORK_IS_UP.store(true, Ordering::SeqCst);
                }
            }else {
                //net down
                //check status
                if was_net_up{
                    notif_send!("network is down");
                    log_error!("network is down");
                    NETWORK_IS_UP.store(false, Ordering::SeqCst);
                } 
            }
        }
        Err(e)=>{
            eprintln!("Failed to execute ping command:{}",e);
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);
            if was_net_up {
                let massage = format!("network connection lost:{}",e);
                notif_send!("{}",massage);
                log_error!("network connection lost");
                NETWORK_IS_UP.store(false, Ordering::SeqCst);
            }
            return Err(e);
        }
    } 
    Ok(())  
}
