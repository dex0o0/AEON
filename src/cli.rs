mod daemon{
    pub mod daemon;
    pub mod notif;
    pub mod log;
}
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell,generate};
use std::{env, fs, io::{self, Write}, path::PathBuf, process::Command as bash, str::FromStr};
use serde::{Serialize,Deserialize};
use crate::daemon::log::Log;

//consts values
const CONF_DIR:&str="/tmp/AEON/system";
const FILE_DATA_PATH:&str="/tmp/AEON/system/config.json";

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
    #[command(about="controle service")]
    Srv{
        #[command(subcommand)]
        action:Config,
    },
    #[command(about="config auto sugestiones")]
    Complation{
        shell:Shell,
    }
}

#[derive(Subcommand,Deserialize,Serialize)]
enum Config{
    #[command(about="set cputreshold for sed notification")]
    Cputsh{
        #[arg(help="set zone for send notif (CPUtreshold)")]
        value:u8,
    },
    #[command(about="show status service")]
    Status,
    #[command(about="restarted service")]
    Restart,
    #[command(about="started service")]
    Start,
    #[command(about="stoped service")]
    Stop,
}

#[derive(Deserialize,Serialize,Debug)]
pub struct DataConf{
    cputsh:Option<f32>,
}

fn get_data(){
    let path = PathBuf::from_str(FILE_DATA_PATH).expect("Error to convert path");
    create_dir_conf(&path);
}

fn create_dir_conf(path:&PathBuf){
    let path_dir = path.parent().expect("Error parent");
    if !path_dir.exists(){
        let _ = fs::create_dir_all(path_dir).map_err(|e| Log::save_log("ERRORS", format!("can't create dir config:{}",e)));   
    }
    if !path.exists(){
        let _ = fs::File::create(path).map_err(|e| Log::save_log("ERRORS", format!("can't create file config:{}",e)));
    }
}

fn read_data(path:&PathBuf)-> Option<DataConf>{
   let data = fs::read_to_string(path).expect("ERRORS");
   if let Ok(json) = serde_json::from_slice(data.as_bytes()){
       Some(json)
   }else {
       None
   }
   
}


async fn save_conf(data:DataConf){
    let homedir= env::home_dir().expect("Error homedir");
    let conf_path = homedir.join(".config/AEON/config.json");
    let mut file = fs::File::create(&conf_path).expect("Error");
    let json = serde_json::to_string_pretty(&data).expect("Error");
    file.write_all(json.as_bytes()).expect("Error can not write data");
}

#[tokio::main]
async fn main()->io::Result<()>{
    let homedir= env::home_dir().expect("Error homedir");
    let conf_path = homedir.join(".config/AEON/config.json");
    create_dir_conf(&conf_path);
    let cli = Cli::parse();
    match cli.command{
        Command::Srv{action}=>{
            match action {
                Config::Cputsh{value}=>{
                    let conf = DataConf{
                        cputsh:Some(value as f32)
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
        Command::Complation { shell }=>{
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
    }
   Ok(()) 
}


