use crate::daemon::{log::Log, notif::Notif};
use sysinfo::{Disks, System};

pub async fn monswap(){
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let swap = sysinfo::System::free_swap(&sys);
    let total = sysinfo::System::total_swap(&sys);
    if total == 0{
        return;
    }
    if (total - swap) as f32 >= total as f32 * 0.8 {
        let massage = "your use of partition swap is high\nplease check".to_string();
        let _ =Notif::send("AEON", massage);
    }
} 
pub async fn moncpu(value:f32){
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    if sys.global_cpu_usage() > value{
        let _ = Log::save_log("System", format!("cpu usage:{}",sys.global_cpu_usage()));
        let massage = format!("CPU very high:{}%",sys.global_cpu_usage());
        let _ = Notif::send("CPU", massage);
    }
}
pub async fn gpu(){
    let sys = System::new_all();
    if sys.global_cpu_usage() > 80.0{
        let _ = Log::save_log("System", format!("gpu usage:{}",sys.global_cpu_usage()));
        let massage = format!("GPU Usage:{}",sys.global_cpu_usage());
        let _ = Notif::send("AEON", massage);
    }
}
pub async fn check_disk(){
    let disks = Disks::new_with_refreshed_list();
    disks.iter().for_each(|disk| {
        let total = disk.total_space();
        let free_space = disk.available_space();
        let use_space = total - free_space;
        let zone90 = total as f32 * 0.9;
        let montpoint = disk.mount_point().display();
        if use_space as f32 >= zone90{
            let masssage = format!("storage space filling\n\
                disk\ttotal\tusage\tfree\n\
                {}\t{:.2}G\t{:.2}G\t{:.2}G\t{}",
                disk.name().to_string_lossy(),
                (total as f32 / 1024.0/1024.0/1024.0),
                (use_space as f32 /1024.0/1024.0/1024.0),
                (free_space as f32 /1024.0/1024.0/1024.0),
                montpoint);
            let _ = Log::save_log("disk", masssage);
            let _ = Notif::send("DISK", format!("disk:{},is filling please check",disk.name().to_string_lossy()));
        }
    }); 
}
pub async fn check_mem(){
    let sys = System::new_all();
    // sys.refresh_memory();
    let total = sys.total_memory();
    let usage = sys.used_memory();
    
    if usage as f32 >= (total as f32 * 80.0){
        let massage = format!("mempry usage is very high:{}",(usage as f32/1024.0/1024.0/1024.0));
        let _ = Log::save_log("System", massage.clone());
        let _ = Notif::send("MEMORY", massage);
    }
}
