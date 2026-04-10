use std::{
    fs, io::{self, Read}, path::PathBuf, process::Command, sync::atomic::{AtomicBool,Ordering}

};
use crate::{daemon::{log::{self, Log}, notif::Notif}, modules::monitoring};



static NETWORK_IS_UP:AtomicBool=AtomicBool::new(true);


pub async fn check_net()-> io::Result<()>{
    let ping = Command::new("ping")
        .args(["-W 5","-c 1","8.8.8.8"])
        .output();
    match ping {
        Ok(output)=>{
            let was_net_up = NETWORK_IS_UP.load(Ordering::SeqCst);
            
            if output.status.success(){
                //net up 
                //
                //check status
                if !was_net_up{
                    let _ = Notif::send("NETWORK", "back to online".to_string());
                    let _ = log::senderror("network up");
                    NETWORK_IS_UP.store(true, Ordering::SeqCst);
                }
            }else {
                //net down
                //
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

pub async fn file_chek(path:PathBuf)-> String{
    read_file(&path).map(|data| data).expect("Err")
}
pub fn read_file(path:&PathBuf)-> Option<String>{
    let mut file = fs::File::open(path).expect("Err");
    let mut data = String::new();
    if file.read_to_string(&mut data).is_ok(){
        let json = serde_json::to_string_pretty(&data).expect("Error");
        println!("json:{}",json);
        Some(json)
    }else {
        None
    }
    

}
