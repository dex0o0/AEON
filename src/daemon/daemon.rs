use std::{
    process::Command
};
use crate::{daemon::notif::Notif, modules::monitoring};



pub fn check_swap(){
    monitoring::monswap();
}
pub async fn check_cpu(){
    monitoring::moncpu().await;
}
pub async fn check_net(){
    let ping = Command::new("ping")
        .args(["-W 10","-c 1","8.8.8.8"])
        .output().expect("Error");
    if !ping.status.success(){
        let _ = Notif::send("Network", "we can't connect to network".to_string());
    }
}
