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
        let cmd = self.command.as_ref().unwrap_or(&self.name);
        let m1 = self
            .pidfile
            .as_ref()
            .and_then(|f| read_to_string(f).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .and_then(|p| procs.service_by_pid(&p));
        let mut m2 = procs.services_by_cmd(cmd);
        m2.sort();
        if let Some(m1) = m1 {
            found.push((&self.name, m1, String::from("")));
        } else if m2.is_empty() && self.required {
            found.push((&self.name, 0, String::from("service not running")));
        } else if !m2.is_empty() {
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
        found
    }
}

pub struct Config(Vec<ConfigItem>);

impl Config {
    pub fn new() -> Result<Config, Error> {
        let mut items = vec![];
        for path in locate()? {
            for line in read_to_string(path)?.lines() {
                let mut parts = line.split_whitespace();
                let title = parts.next().unwrap_or("");
                let pidfile = parts.next().map(|s| s.to_string()).filter(|s| s != "-");
                let cmd = parts.collect::<Vec<&str>>().join(" ");
                if !title.is_empty() && !title.starts_with("#") {
                    let name = title.trim_start_matches(['-', '+', '*']).to_string();
                    let required = !title.starts_with(['-', '*']);
                    let multiple = title.starts_with(['+', '*']);
                    let command = (!cmd.is_empty()).then_some(cmd);
                    items.push(ConfigItem { name, required, multiple, pidfile, command });
                }
            }
        }
        Ok(Config(items))
    }

    pub fn empty() -> Config {
        Config(vec![])
    }

    pub fn all(&self, procs: &ProcessMap) -> Vec<(&String, u32, String)> {
        self.0.iter().flat_map(|item| item.matches(procs)).collect()
    }
}

fn missing(msg: &str) -> Error {
    Error::new(ErrorKind::NotFound, msg)
}

fn locate() -> Result<Vec<PathBuf>, Error> {
    if let Ok(str) = env::var("UPSTATE_CONF") {
        let path = Path::new(&str).to_path_buf();
        if path.is_file() {
            return Ok(vec![path]);
        } else if path.is_dir() {
            return locate_files(path);
        } else {
            return Err(missing(&format!("config file not found: {}", str)));
        }
    }
    let alt1 = env::current_exe()?;
    let alt2 = env::current_dir()?;
    for dir in alt1.ancestors().skip(1).chain(alt2.ancestors()) {
        let path = dir.join("upstate.conf");
        if path.is_file() {
            return Ok(vec![path]);
        }
        let path = dir.join("etc/upstate.conf");
        if path.is_file() {
            return Ok(vec![path]);
        }
        let path = dir.join("etc/upstate.conf.d");
        if path.is_dir() {
            return locate_files(path);
        }
    }
    Err(missing("no upstate.conf file found"))
}

fn locate_files(path: PathBuf) -> Result<Vec<PathBuf>, Error> {
    let paths = path.read_dir()?.filter_map(|e| e.ok()).map(|e| e.path());
    let mut files: Vec<_> = paths.filter(|p| p.is_file()).collect();
    files.sort();
    Ok(files)
}
