use crate::daemon::{log::Log, notif::Notif};
use sysinfo::System;




pub fn monswap(){
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
pub async fn moncpu(){
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    if sys.global_cpu_usage() > (100.0 * 0.8){
        let _ = Log::save_log("System", format!("cpu usage:{}",sys.global_cpu_usage()));
        let massage = format!("CPU very high:{}%",sys.global_cpu_usage());
        let _ = Notif::send("CPU", massage);
    }
}
pub async fn gpu(){
    let sys = System::new_all();
    if sys.global_cpu_usage() > (100.0 * 0.8){
        let _ = Log::save_log("System", format!("gpu usage:{}",sys.global_cpu_usage()));
        let massage = format!("GPU Usage:{}",sys.global_cpu_usage());
        let _ = Notif::send("AEON", massage);
    }

}
