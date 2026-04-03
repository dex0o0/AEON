use chrono::{DateTime,Local};
use std::{
    env, fs, io::Write, path::PathBuf
};

pub struct Log{
    pub head:String,
    pub body:String,
}


impl Log {
    pub fn new(head:&'static str ,body:&'static str)->Self{
        Self{
            head:head.to_string(),
            body:body.to_string(),
        }
    }
    pub fn save_log(self)-> std::io::Result<()>{
        let homedir = env::home_dir().unwrap_or_else(||{
            eprintln!("Error");
            let _ = senderror("ERROR_from");
            PathBuf::from("/tmp/")
        });
        let time:DateTime<Local> = Local::now(); 
        let path_log = format!("{}/.log/dex_daemon/{}-{}.log",homedir.display(),self.head,time.date_naive());
        let pathlog = PathBuf::from(path_log.clone());
        let dirlog = format!("{}/.log/dex_daemon/",homedir.display());
        if !PathBuf::from(&dirlog).exists(){
            let _ = fs::create_dir_all(PathBuf::from(&dirlog));
            let _ = fs::File::create(&pathlog);
        }
        let mut log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path_log)?;
        let _ = writeln!(log,"{}",self.body);
        Ok(())
    }
}

fn senderror(noties:&'static str)-> std::io::Result<()>{
    let mut logerror=fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/ERROR-dex.log")?;
    let _ =writeln!(logerror,"{}",noties);
    Ok(())
}


