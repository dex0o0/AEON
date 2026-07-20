use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
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

pub fn senderror(noties: &'static str) -> Result<()> {
    let mut logerror = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/ERROR-dex.log")?;

    let time: DateTime<Local> = Local::now();
    let massage = format!("{}: {}", time, noties);
    let _ = writeln!(logerror, "{}", massage);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logstatus {
    pub path: PathBuf,
    pub line_num: Vec<u64>,
    pub level: Loglevel,
    pub prob: String,
    pub count: u64,
    pub f_time: DateTime<Local>,
    pub l_time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Loglevel {
    Good,
    Normal,
    Warning,
    Critical,
}

impl Default for Logstatus {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            line_num: vec![],
            level: Loglevel::Good,
            prob: "".to_string(),
            count: 0u64,
            f_time: Local.timestamp_opt(0, 0).unwrap(),
            l_time: Local.timestamp_opt(0, 0).unwrap(),
        }
    }
}
impl Logstatus {
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path,
            ..Self::default()
        }
    }
    pub fn analyse(&mut self) -> std::io::Result<()> {
        if !self.path.exists() {
            self.level = Loglevel::Good;
            self.prob = "No log file found.System clean".to_string();
            return Ok(());
        }
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut level = Loglevel::Good;
        let mut msg = String::new();

        for (c_line, line) in reader.lines().enumerate() {
            if let Ok(log_line) = line {
                let mut danger = false;
                if log_line.contains("CRITICAL")
                    || log_line.contains("panic")
                    || log_line.contains("down")
                {
                    level = std::cmp::max(level, Loglevel::Critical);
                    danger = true;
                } else if log_line.contains("WARN") || log_line.contains("high") {
                    level = std::cmp::max(level, Loglevel::Warning);
                    danger = true;
                } else if log_line.contains("INFO") || log_line.contains("SYSTEM") {
                    level = std::cmp::max(level, Loglevel::Normal);
                }
                if danger {
                    self.line_num.push(c_line as u64);
                    self.count += 1;
                    if !msg.is_empty() {
                        msg.push('\n');
                    }
                    msg.push_str(&log_line);

                    if let Some(pos) = log_line.find(": ") {
                        let time_str = &log_line[0..pos];

                        if let Ok(parsed_time) =
                            DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S%.f %z")
                        {
                            let local_time: DateTime<Local> = DateTime::from(parsed_time);

                            if self.count == 1 {
                                self.f_time = local_time;
                            }
                            self.l_time = local_time;
                        }
                    }
                }
            }
        }
        self.level = level;
        self.prob = msg;
        Ok(())
    }
}
