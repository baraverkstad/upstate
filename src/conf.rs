use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::proc::ProcessMap;

pub struct ConfigItem {
    name: String,
    required: bool,
    pidfile: Option<String>,
    command: Option<String>,
}

pub struct Config(Vec<ConfigItem>);

impl Config {
    pub fn new() -> Result<Config, Error> {
        let filename = locate()?;
        let mut items = vec![];
        for line in fs::read_to_string(filename)?.lines() {
            let mut parts = line.split_whitespace();
            let title = parts.next().unwrap_or("");
            let pidfile = parts.next().map(|s| s.to_string()).filter(|s| s != "-");
            let cmd = parts.collect::<Vec<&str>>().join(" ");
            if !title.is_empty() && !title.starts_with("#") {
                let required = !title.starts_with("-");
                let name = title.trim_start_matches("-").to_string();
                let command = (cmd.len() > 0).then(|| cmd);
                items.push(ConfigItem { name, required, pidfile, command });
            }
        }
        return Ok(Config(items));
    }

    pub fn empty() -> Config {
        return Config(vec![]);
    }

    pub fn all(&self, procs: &ProcessMap) -> Vec<(&String, u32, String)> {
        return self
            .0
            .iter()
            .map(|item| -> (&String, u32, String) {
                let cmd = item.command.as_ref().or(Some(&item.name)).unwrap();
                let m1 = item
                    .pidfile
                    .as_ref()
                    .and_then(|f| fs::read_to_string(f).ok())
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .and_then(|p| procs.service_by_pid(&p));
                let m2 = procs.service_by_cmd(cmd);
                if m1.is_some() {
                    return (&item.name, m1.unwrap(), String::from(""));
                } else if m2.is_some() {
                    let mut msg = String::from("");
                    if item.pidfile.is_some() {
                        msg = format!("invalid PID file {}", item.pidfile.as_ref().unwrap());
                    }
                    return (&item.name, m2.unwrap(), msg);
                } else {
                    let msg = if item.required { "service not running" } else { "" };
                    return (&item.name, 0, String::from(msg));
                }
            })
            .collect();
    }
}

fn missing(msg: &str) -> Error {
    Error::new(ErrorKind::NotFound, msg)
}

fn validpath(path: PathBuf) -> Option<PathBuf> {
    path.exists().then_some(path)
}

fn locate() -> Result<PathBuf, Error> {
    if let Ok(str) = env::var("UPSTATE_CONF") {
        return validpath(Path::new(&str).to_path_buf())
            .ok_or_else(|| missing(&format!("config file not found: {}", str)));
    }
    let alt1 = env::current_exe()?;
    let alt2 = env::current_dir()?;
    for dir in alt1.ancestors().skip(1).chain(alt2.ancestors()) {
        let p1 = validpath(dir.join("upstate.conf"));
        let p2 = validpath(dir.join("etc/upstate.conf"));
        if let Some(cfg) = p1.or(p2) {
            return Ok(cfg);
        }
    }
    return Err(missing("no upstate.conf file found"));
}
