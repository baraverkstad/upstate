use std::env;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::proc::ProcessMap;

pub struct ConfigItem {
    name: String,
    required: bool,
    multiple: bool,
    pidfile: Option<String>,
    command: Option<String>,
}

impl ConfigItem {
    pub fn matches(&self, procs: &ProcessMap) -> Vec<(&String, u32, String)> {
        let mut found = vec![];
        let cmd = self.command.as_ref().or(Some(&self.name)).unwrap();
        let m1 = self
            .pidfile
            .as_ref()
            .and_then(|f| read_to_string(f).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .and_then(|p| procs.service_by_pid(&p));
        let mut m2 = procs.services_by_cmd(cmd);
        m2.sort();
        if m1.is_some() {
            found.push((&self.name, m1.unwrap(), String::from("")));
        } else if m2.len() == 0 && self.required {
            found.push((&self.name, 0, String::from("service not running")));
        } else if m2.len() > 0 {
            let mut msg = String::from("");
            if self.pidfile.is_some() {
                msg = format!("invalid PID file {}", self.pidfile.as_ref().unwrap());
            } else if m2.len() > 1 && !self.multiple {
                msg = String::from("multiple matching processes");
            }
            for pid in m2 {
                found.push((&self.name, pid, msg.clone()));
            }
        }
        return found;
    }
}

pub struct Config(Vec<ConfigItem>);

impl Config {
    pub fn new() -> Result<Config, Error> {
        let filename = locate()?;
        let mut items = vec![];
        for line in read_to_string(filename)?.lines() {
            let mut parts = line.split_whitespace();
            let title = parts.next().unwrap_or("");
            let pidfile = parts.next().map(|s| s.to_string()).filter(|s| s != "-");
            let cmd = parts.collect::<Vec<&str>>().join(" ");
            if !title.is_empty() && !title.starts_with("#") {
                let name = title.trim_start_matches(&['-', '+', '*']).to_string();
                let required = !title.starts_with(&['-', '*']);
                let multiple = title.starts_with(&['+', '*']);
                let command = (cmd.len() > 0).then(|| cmd);
                items.push(ConfigItem { name, required, multiple, pidfile, command });
            }
        }
        return Ok(Config(items));
    }

    pub fn empty() -> Config {
        return Config(vec![]);
    }

    pub fn all(&self, procs: &ProcessMap) -> Vec<(&String, u32, String)> {
        return self.0.iter().flat_map(|item| item.matches(&procs)).collect();
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
