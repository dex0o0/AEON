use aeon::daemon::log::Log;
use aeon::modules;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::Command as bash,
};

//consts values
// const CONF_DIR:&str="/tmp/AEON/system";
// const FILE_DATA_PATH:&str="/tmp/AEON/system/config.json";

//cli conf
#[derive(Parser)]
#[command(name = "aeoncli")]
#[command(version = "0.1.24")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "controle service")]
    Srv {
        #[command(subcommand)]
        action: Config,
    },
    #[command(short_flag = 'b', about = "Backup files and directories")]
    Backup {
        #[arg(short, long)]
        path: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short, long, default_value = "gzip")]
        compress: Compression,

        #[arg(short, long, default_value_t = 6)]
        level: u8,

        #[arg(short, long)]
        exclude: Vec<String>,

        #[arg(short, long)]
        verbose: bool,
    },

    #[command(about = "config auto sugestiones")]
    Complation { shell: Shell },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Compression {
    Gzip,
    Bzip2,
    Xz,
    None,
}

#[derive(Subcommand, Deserialize, Serialize)]
pub enum Config {
    #[command(about = "set cputreshold for sed notification")]
    Cputsh {
        #[arg(help = "set zone for send notif (CPUtreshold)")]
        value: u8,
    },
    #[command(about = "show status service")]
    Status,
    #[command(about = "restarted service")]
    Restart,
    #[command(about = "started service")]
    Start,
    #[command(about = "stoped service")]
    Stop,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DataConf {
    pub cputsh: Option<f32>,
}

// fn get_data(){
//     let path = PathBuf::from_str(FILE_DATA_PATH).expect("Error to convert path");
//     create_dir_conf(&path);
// }

#[allow(unused)]
fn create_dir_conf(path: &PathBuf) {
    let path_dir = path.parent().expect("Error parent");
    if !path_dir.exists() {
        let _ = fs::create_dir_all(path_dir)
            .map_err(|e| Log::save_log("ERRORS", format!("can't create dir config:{}", e)));
    }
    if !path.exists() {
        let _ = fs::File::create(path)
            .map_err(|e| Log::save_log("ERRORS", format!("can't create file config:{}", e)));
    }
}

#[allow(unused)]
async fn save_conf(data: DataConf) {
    let homedir = env::home_dir().expect("Error homedir");
    let conf_path = homedir.join(".config/AEON/config.json");
    let mut file = fs::File::create(&conf_path).expect("Error");
    let json = serde_json::to_string_pretty(&data).expect("Error");
    file.write_all(json.as_bytes())
        .expect("Error can not write data");
}

#[allow(unused)]
#[tokio::main]
async fn main() -> io::Result<()> {
    let homedir = env::home_dir().expect("Error homedir");
    let conf_path = homedir.join(".config/AEON/config.json");
    create_dir_conf(&conf_path);
    let cli = Cli::parse();
    match cli.command {
        Command::Srv { action } => match action {
            Config::Cputsh { value } => {
                let conf = DataConf {
                    cputsh: Some(value as f32),
                };
                save_conf(conf).await;
            }
            Config::Status => {
                let out = bash::new("systemctl")
                    .args(["status", "AEON.service"])
                    .output()
                    .expect("can not run code");

                println!("{}", String::from_utf8_lossy(&out.stdout));
            }
            Config::Start => {
                let out = bash::new("systemctl")
                    .args(["start", "AEON.service"])
                    .output()
                    .expect("can not run code");

                println!("{}", String::from_utf8_lossy(&out.stdout));
            }
            Config::Restart => {
                let out = bash::new("systemctl")
                    .args(["restart", "AEON.service"])
                    .output()
                    .expect("can not run code");

                println!("{}", String::from_utf8_lossy(&out.stdout));
            }
            Config::Stop => {
                let out = bash::new("systemctl")
                    .args(["stop", "AEON.service"])
                    .output()
                    .expect("can not run code");

                println!("{}", String::from_utf8_lossy(&out.stdout));
            }
        },
        Command::Complation { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Command::Backup {
            path,
            output,
            compress,
            level,
            exclude,
            verbose,
        } => {
            let compression = match compress {
                Compression::Gzip => modules::backup::CompressionType::Gzip,
                Compression::Bzip2 => modules::backup::CompressionType::Bzip2,
                Compression::Xz => modules::backup::CompressionType::Xz,
                Compression::None => modules::backup::CompressionType::None,
            };

            // Determine output directory
            let output_dir = output.unwrap_or_else(|| {
                std::env::current_dir().expect("Failed to get current directory")
            });

            // Create config
            let config = modules::backup::BackupConfig {
                source_path: path,
                output_dir,
                compression,
                compression_level: level,
                exclude_patterns: exclude,
            };

            // Run backup
            if verbose {
                println!("Starting backup...");
                println!("Source: {}", config.source_path.display());
                println!("Output: {}", config.output_dir.display());
                println!("Compression: {:?}", config.compression);
            }

            match modules::backup::create_backup(config) {
                Ok(_) => {
                    println!("Backup completed successfully!");
                }
                Err(e) => {
                    eprintln!("Backup failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
