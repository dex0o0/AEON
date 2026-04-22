use chrono::{DateTime, Local};
use std::{env, fs, io::Write, path::PathBuf};

pub struct Log;

impl Log {
    pub fn save_log(head: &'static str, body: String) -> std::io::Result<()> {
        let homedir = env::home_dir().unwrap_or_else(|| {
            eprintln!("Error");
            let _ = senderror("ERROR_from get home dir");
            PathBuf::from("/tmp/")
        });
        let path_log = format!("{}/.log/dex_daemon/{}.log", homedir.display(), head);
        let pathlog = PathBuf::from(path_log.clone());
        let dirlog = format!("{}/.log/dex_daemon/", homedir.display());
        if !PathBuf::from(&dirlog).exists() {
            let _ = fs::create_dir_all(PathBuf::from(&dirlog));
            let _ = fs::File::create(&pathlog);
        }
        let mut log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path_log)?;

        let time: DateTime<Local> = Local::now();
        let massage = format!("{}: {}", time, body);

        let _ = writeln!(log, "{}", massage);
        Ok(())
    }
}

pub fn senderror(noties: &'static str) -> std::io::Result<()> {
    let mut logerror = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/ERROR-dex.log")?;

    let time: DateTime<Local> = Local::now();
    let massage = format!("{}: {}", time, noties);
    let _ = writeln!(logerror, "{}", massage);
    Ok(())
}
