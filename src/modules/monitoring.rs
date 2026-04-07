use crate::daemon::{log::{self, Log}, notif::{self, Notif}};
use sysinfo::{Cpu, System};




pub fn monswap(){
    let mut sys = System::new_all();
    sys.refresh_all();

    let swap = sysinfo::System::free_swap(&sys);
    let total = sysinfo::System::total_swap(&sys);
    if (total - swap) as f32 >= total as f32 * 0.8 {
        let massage = "your use of partition swap is high\nplease check".to_string();
        let _ =Notif::send("AEON", massage);
    }

    
} 
pub async fn moncpu(){
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    if sys.global_cpu_usage() > (100.0 * 0.8){
        let _ = Log::save_log("cpu usage", format!("usage:{}",sys.global_cpu_usage()));
        let massage = format!("CPU very high:{}%",sys.global_cpu_usage());
        let _ = Notif::send("CPU", massage);
    }

} 
