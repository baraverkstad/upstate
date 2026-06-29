use std::env;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::proc::ProcessMap;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Service {
    name: String,
    #[serde(default = "default_true")]
    required: bool,
    #[serde(default)]
    multiple: bool,
    pidfile: Option<String>,
    command: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Service {
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
                msg = format!("invalid PID file {}", self.pidfile.as_deref().unwrap_or("?"));
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

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    services: Vec<Service>,
}

impl Config {
    pub fn new() -> Result<Config, Error> {
        let mut merged = Config::empty();
        for path in locate()? {
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                merged.services.append(&mut parse_toml(&path)?.services);
            } else {
                merged.services.extend(parse_legacy(&path)?);
            }
        }
        Ok(merged)
    }

    pub fn empty() -> Config {
        Config { services: vec![] }
    }

    pub fn service_matches(&self, procs: &ProcessMap) -> Vec<(&String, u32, String)> {
        self.services.iter().flat_map(|item| item.matches(procs)).collect()
    }
}

fn parse_legacy(path: &Path) -> Result<Vec<Service>, Error> {
    let mut items = vec![];
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
            items.push(Service { name, required, multiple, pidfile, command });
        }
    }
    Ok(items)
}

fn parse_toml(path: &Path) -> Result<Config, Error> {
    let data = read_to_string(path)?;
    toml::from_str(&data).map_err(|e| Error::new(ErrorKind::InvalidData, e))
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
        let path = dir.join("upstate.toml");
        if path.is_file() {
            return Ok(vec![path]);
        }
        let path = dir.join("etc/upstate.toml");
        if path.is_file() {
            return Ok(vec![path]);
        }
        let path = dir.join("upstate.conf");
        if path.is_file() {
            return Ok(vec![path]);
        }
        let path = dir.join("etc/upstate.conf");
        if path.is_file() {
            return Ok(vec![path]);
        }
        let path = dir.join("etc/upstate.toml.d");
        if path.is_dir() {
            return locate_files(path);
        }
        let path = dir.join("etc/upstate.conf.d");
        if path.is_dir() {
            return locate_files(path);
        }
    }
    Err(missing("no upstate config found"))
}

fn locate_files(path: PathBuf) -> Result<Vec<PathBuf>, Error> {
    let paths = path.read_dir()?.filter_map(|e| e.ok()).map(|e| e.path());
    let mut files: Vec<_> = paths.filter(|p| p.is_file()).collect();
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn etc_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("etc")
    }

    fn parse_legacy_path(path: &Path) -> Vec<Service> {
        parse_legacy(path).unwrap()
    }

    fn parse_toml_path(path: &Path) -> Vec<Service> {
        parse_toml(path).unwrap().services
    }

    #[test]
    fn test_00_system_default_roundtrip() {
        let legacy = etc_dir().join("upstate.conf.d/00-system-default");
        let toml = etc_dir().join("upstate.toml.d/00-system-default.toml");
        let legacy_items = parse_legacy_path(&legacy);
        let toml_items = parse_toml_path(&toml);
        assert_eq!(legacy_items, toml_items, "00-system-default mapping mismatch");
    }

    #[test]
    fn test_10_system_required_roundtrip() {
        let legacy = etc_dir().join("upstate.conf.d/10-system-required");
        let toml = etc_dir().join("upstate.toml.d/10-system-required.toml");
        let legacy_items = parse_legacy_path(&legacy);
        let toml_items = parse_toml_path(&toml);
        assert_eq!(legacy_items, toml_items, "10-system-required mapping mismatch");
    }

    #[test]
    fn test_20_docker_roundtrip() {
        let legacy = etc_dir().join("upstate.conf.d/20-docker");
        let toml = etc_dir().join("upstate.toml.d/20-docker.toml");
        let legacy_items = parse_legacy_path(&legacy);
        let toml_items = parse_toml_path(&toml);
        assert_eq!(legacy_items, toml_items, "20-docker mapping mismatch");
    }

    #[test]
    fn test_90_local_roundtrip() {
        let legacy = etc_dir().join("upstate.conf.d/90-local");
        let toml = etc_dir().join("upstate.toml.d/90-local.toml");
        let legacy_items = parse_legacy_path(&legacy);
        let toml_items = parse_toml_path(&toml);
        assert_eq!(legacy_items, toml_items, "90-local mapping mismatch");
    }
}
