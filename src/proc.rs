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
        let mut roots = vec![];
        let mut parents = HashMap::new();
        let mut children = HashMap::new();
        let mut info = HashMap::new();
        for (pid, proc) in sys.processes() {
            if proc.thread_kind().is_none() {
                children.entry(pid.as_u32()).or_insert(vec![]);
                let cmd = proc.cmd().join(OsStr::new(" ")).to_string_lossy().into_owned();
                let rss = proc.memory();
                let cpu = proc.accumulated_cpu_time() / 1000;
                info.insert(pid.as_u32(), ProcessInfo { cmd, cpu, rss });
                if let Some(ppid) = proc.parent() {
                    parents.insert(pid.as_u32(), ppid.as_u32());
                    children.entry(ppid.as_u32()).or_insert(vec![]).push(pid.as_u32());
                } else {
                    roots.push(pid.as_u32());
                }
            }
        }
        ProcessMap { roots, parents, children, info }
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
            pids.extend(self.children.get(pid).unwrap());
        }
        pids
    }

    pub fn service_by_pid(&self, pid: &u32) -> Option<u32> {
        self.info.contains_key(pid).then(|| self.as_service(pid))
    }

    pub fn services_by_cmd(&self, cmd: &str) -> Vec<u32> {
        let re = RegexBuilder::new(cmd).case_insensitive(true).build().unwrap();
        let is_match = |s: &String| s.contains(cmd) || re.is_match(s);
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
