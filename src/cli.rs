mod daemon{
    pub mod daemon;
    pub mod notif;
    pub mod log;
}
use clap::{Parser,Subcommand};

use std::{fs, io::{self, Write}, path::{PathBuf}, str::FromStr,process::Command as bash};
use serde::{Serialize,Deserialize};
use crate::daemon::log::Log;

//consts
const CONF_DIR:&str="/tmp/AEON/system";
const FILE_DATA_PATH:&str="/tmp/AEON/system/config.json";
//

//cli conf
#[derive(Parser)]
#[command(name = "aeoncli")]
#[command(version = "0.1.0")]
struct Cli{
    #[command(subcommand)]
    command:Command,
}

#[derive(Subcommand)]
enum Command{

    Srv{
        #[command(subcommand)]
        action:Config,
    },
}

#[derive(Subcommand,Deserialize,Serialize)]
enum Config{
    Cputsh{
        #[arg(help="")]
        value:u8,
    },
    Status,
    Restart,
    Start,
    Stop,
}
//

#[derive(Deserialize,Serialize,Debug)]
pub struct DataConf{
    cputsh:Option<u8>,
}

fn get_data(){
    let path = PathBuf::from_str(FILE_DATA_PATH).expect("Error to convert path");
    create_dir_conf(&path);
}

fn create_dir_conf(path:&PathBuf){
    let path_dir = PathBuf::from_str(CONF_DIR).expect("Error");
    if !path.parent().expect("ERRORS:get parent path config").exists(){
        let _ = fs::create_dir_all(path_dir).map_err(|e| Log::save_log("ERRORS", format!("can't create dir config:{}",e)));   
    }
    if !path.exists(){
        let _ = fs::File::create(path).map_err(|e| Log::save_log("ERRORS", format!("can't create file config:{}",e)));
    }
}

fn read_data(path:&PathBuf)-> Option<Config>{
   let data = fs::read_to_string(path).expect("ERRORS");
   if let Ok(json) = serde_json::from_slice(data.as_bytes()){
       Some(json)
   }else {
       None
   }
   
}


async fn save_conf(data:DataConf){
    let path = PathBuf::from_str(FILE_DATA_PATH).expect("ERRORS");
    let mut file = fs::File::create(path).expect("Error");
    let json = serde_json::to_string_pretty(&data).expect("Error");
    file.write_all(json.as_bytes()).expect("Error can not write data");
}

#[tokio::main]
async fn main()->io::Result<()>{
    let conf = DataConf{
        cputsh:Some(32),
    };
    create_dir_conf(&PathBuf::from_str(FILE_DATA_PATH).expect("Error"));
    save_conf(conf).await;
    let cli = Cli::parse();
    match cli.command{
        Command::Srv{action}=>{
            match action {
                Config::Cputsh{value}=>{
                    let conf = DataConf{
                        cputsh:Some(value)
                    };
                    save_conf(conf).await;
                },
                Config::Status=>{
                    let out = bash::new("systemctl")
                        .args(["status","AEON.service"])
                        .output().expect("can not run code");

                    println!("{}",String::from_utf8_lossy(&out.stdout));
                },
                Config::Start=>{
                    let out = bash::new("systemctl")
                        .args(["start","AEON.service"])
                        .output().expect("can not run code");

                    println!("{}",String::from_utf8_lossy(&out.stdout));
                },
                Config::Restart=>{
                    let out = bash::new("systemctl")
                        .args(["restart","AEON.service"])
                        .output().expect("can not run code");

                    println!("{}",String::from_utf8_lossy(&out.stdout));
                },
                Config::Stop=>{
                    let out = bash::new("systemctl")
                        .args(["stop","AEON.service"])
                        .output().expect("can not run code");

                    println!("{}",String::from_utf8_lossy(&out.stdout));

                },

            }
        },
    }
   Ok(()) 
}


