use regex::RegexBuilder;
use std::collections::HashMap;
use std::ffi::OsStr;
use sysinfo::System;

pub struct ProcessInfo {
    cmd: String,
    cpu: u64,
    rss: u64,
}

pub struct ProcessMap {
    roots: Vec<u32>,
    parents: HashMap<u32, u32>,
    children: HashMap<u32, Vec<u32>>,
    info: HashMap<u32, ProcessInfo>,
}

impl ProcessMap {
    pub fn new(sys: &System) -> ProcessMap {
        let mut map = ProcessMap {
            roots: vec![],
            parents: HashMap::new(),
            children: HashMap::new(),
            info: HashMap::new(),
        };
        for (pid, proc) in sys.processes() {
            if proc.thread_kind().is_none() {
                let cmd = proc.cmd().join(OsStr::new(" ")).to_string_lossy().into_owned();
                let rss = proc.memory();
                let cpu = proc.accumulated_cpu_time() / 1000;
                map.insert(pid.as_u32(), proc.parent().map(|parent| parent.as_u32()), cmd, cpu, rss);
            }
        }
        map
    }

    fn insert(&mut self, pid: u32, parent: Option<u32>, cmd: String, cpu: u64, rss: u64) {
        self.children.entry(pid).or_default();
        self.info.insert(pid, ProcessInfo { cmd, cpu, rss });
        if let Some(parent) = parent {
            self.parents.insert(pid, parent);
            self.children.entry(parent).or_default().push(pid);
        } else {
            self.roots.push(pid);
        }
    }

    #[cfg(test)]
    pub(crate) fn from_test_records(records: &[(u32, Option<u32>, &str, u64, u64)]) -> ProcessMap {
        let mut map = ProcessMap {
            roots: vec![],
            parents: HashMap::new(),
            children: HashMap::new(),
            info: HashMap::new(),
        };
        for (pid, parent, cmd, cpu, rss) in records {
            map.insert(*pid, *parent, (*cmd).to_string(), *cpu, *rss);
        }
        map
    }

    fn is_service(&self, pid: &u32) -> bool {
        self.parents.get(pid).is_none_or(|ppid| self.roots.contains(ppid))
    }

    fn as_service(&self, pid: &u32) -> u32 {
        if self.is_service(pid) {
            return *pid;
        }
        if let Some(ppid) = self.parents.get(pid)
            && self.is_service(ppid)
        {
            return *ppid;
        }
        *pid
    }

    pub fn services(&self) -> Vec<u32> {
        let mut pids = vec![];
        for pid in &self.roots {
            if let Some(children) = self.children.get(pid) {
                pids.extend(children);
            }
        }
        pids
    }

    pub fn service_by_pid(&self, pid: &u32) -> Option<u32> {
        self.info.contains_key(pid).then(|| self.as_service(pid))
    }

    pub fn services_by_cmd(&self, cmd: &str) -> Vec<u32> {
        let re = RegexBuilder::new(cmd).case_insensitive(true).build();
        let is_match = |s: &String| s.contains(cmd) || re.as_ref().is_ok_and(|r| r.is_match(s));
        self.info
            .iter()
            .filter(|(_, v)| is_match(&v.cmd))
            .map(|(k, _)| self.as_service(k))
            .collect()
    }

    pub fn stat(&self, pid: &u32) -> (u64, u64) {
        let mut cpu = 0;
        let mut rss = 0;
        if let Some(info) = self.info.get(pid) {
            cpu += info.cpu;
            rss += info.rss;
            if let Some(child_pids) = self.children.get(pid) {
                for cid in child_pids {
                    let (c, r) = self.stat(cid);
                    cpu += c;
                    rss += r;
                }
            }
        }
        (cpu, rss)
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessMap;

    fn processes() -> ProcessMap {
        ProcessMap::from_test_records(&[
            (1, None, "init", 1, 10),
            (2, Some(1), "ApiServer [blue] --port 8000", 2, 20),
            (3, Some(2), "worker", 3, 30),
            (4, Some(3), "worker-child", 4, 40),
            (5, Some(1), "CacheServer", 5, 50),
            (6, None, "orphan-root", 6, 60),
        ])
    }

    #[test]
    fn services_are_root_children_and_pids_promote_to_service() {
        let processes = processes();
        let mut services = processes.services();
        services.sort();
        assert_eq!(services, vec![2, 5]);
        assert_eq!(processes.service_by_pid(&1), Some(1));
        assert_eq!(processes.service_by_pid(&3), Some(2));
        assert_eq!(processes.service_by_pid(&4), Some(4));
        assert_eq!(processes.service_by_pid(&99), None);
    }

    #[test]
    fn command_matching_supports_case_insensitive_regex_and_literal_fallback() {
        let processes = processes();
        let mut regex_matches = processes.services_by_cmd("APISERVER.*8000");
        regex_matches.sort();
        assert_eq!(regex_matches, vec![2]);

        // An invalid regex falls back to a case-sensitive literal substring.
        let mut literal_matches = processes.services_by_cmd("[Server");
        literal_matches.sort();
        assert!(literal_matches.is_empty());
        literal_matches = processes.services_by_cmd("[");
        literal_matches.sort();
        assert_eq!(literal_matches, vec![2]);
        literal_matches = processes.services_by_cmd("ApiServer");
        literal_matches.sort();
        assert_eq!(literal_matches, vec![2]);
    }

    #[test]
    fn stat_aggregates_descendants_and_returns_zero_for_missing_pid() {
        let processes = processes();
        assert_eq!(processes.stat(&2), (9, 90));
        assert_eq!(processes.stat(&99), (0, 0));
    }
}
