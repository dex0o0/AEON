use std::{
    io,process::Command, sync::atomic::{AtomicBool,Ordering}
};
use crate::{
    daemon::{log,notif::Notif} 
};

static NETWORK_IS_UP:AtomicBool=AtomicBool::new(false);

pub async fn check_net(ip:&str)-> io::Result<()>{
    let ping = Command::new("ping")
        .args(["-W 10","-c 2",ip])
        .output();
    match ping {
        Ok(output)=>{
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);
            
            if output.status.success(){
                //net up 
                //check status
                if !was_net_up{
                    let _ = Notif::send("NETWORK", "back to online".to_string());
                    let _ = log::senderror("network up");
                    NETWORK_IS_UP.store(true, Ordering::SeqCst);
                }
            }else {
                //net down
                //check status
                if was_net_up{
                    let _ = Notif::send("NETWORK", "network is down".to_string());
                    let _ = log::senderror("network down");
                    NETWORK_IS_UP.store(false, Ordering::SeqCst);
                } 
            }
        }
        Err(e)=>{
            eprintln!("Failed to execute ping command:{}",e);
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);
            if was_net_up {
                let massage = format!("network connection lost:{}",e);
                let _ = Notif::send("NETWORK", massage);
                let _ = log::senderror("network down");
                NETWORK_IS_UP.store(false, Ordering::SeqCst);
            }
            return Err(e);
        }
    } 
    Ok(())  
}
