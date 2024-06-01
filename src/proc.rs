use procfs;
use regex::RegexBuilder;
use std::collections::HashMap;
use sysinfo::System;

pub struct ProcessMap {
    roots: Vec<u32>,
    parents: HashMap<u32, u32>,
    children: HashMap<u32, Vec<u32>>,
    cmds: HashMap<u32, String>,
}

impl ProcessMap {
    pub fn new(sys: &System) -> ProcessMap {
        let mut roots = vec![];
        let mut parents = HashMap::new();
        let mut children = HashMap::new();
        let mut cmds = HashMap::new();
        for (pid, proc) in sys.processes() {
            children.entry(pid.as_u32()).or_insert(vec![]);
            cmds.insert(pid.as_u32(), proc.cmd().join(" "));
            if let Some(ppid) = proc.parent() {
                parents.insert(pid.as_u32(), ppid.as_u32());
                children.entry(ppid.as_u32()).or_insert(vec![]).push(pid.as_u32());
            } else {
                roots.push(pid.as_u32());
            }
        }
        return ProcessMap { roots, parents, children, cmds };
    }

    fn is_service(&self, pid: &u32) -> bool {
        let ppid = self.parents.get(pid);
        return ppid.is_none() || self.roots.contains(ppid.unwrap());
    }

    fn as_service(&self, pid: &u32) -> u32 {
        if self.is_service(pid) {
            return *pid;
        }
        let ppid = self.parents.get(pid);
        if self.is_service(ppid.unwrap()) {
            return *ppid.unwrap();
        } else {
            return *pid;
        }
    }

    pub fn services(&self) -> Vec<u32> {
        let mut pids = vec![];
        for pid in &self.roots {
            pids.extend(self.children.get(&pid).unwrap());
        }
        return pids;
    }

    pub fn service_by_pid(&self, pid: &u32) -> Option<u32> {
        return self.cmds.contains_key(pid).then(|| self.as_service(pid));
    }

    pub fn service_by_cmd(&self, cmd: &str) -> Option<u32> {
        let re = RegexBuilder::new(cmd).case_insensitive(true).build().unwrap();
        let is_match = |s: &String| s.contains(cmd) || re.is_match(s);
        let found = self.cmds.iter().filter_map(|(k, v)| is_match(v).then(|| k));
        return found.map(|p| self.as_service(p)).min();
    }

    pub fn stat(&self, pid: &u32) -> (u64, u64) {
        let (cputicks, rssbytes) = self.stat_collect(pid);
        return (cputicks / procfs::ticks_per_second(), rssbytes);
    }

    fn stat_collect(&self, pid: &u32) -> (u64, u64) {
        let mut cputicks = 0;
        let mut rssbytes = 0;
        let proc = procfs::process::Process::new(*pid as i32);
        if let Ok(stat) = proc.and_then(|p| p.stat()) {
            cputicks += stat.utime + stat.stime;
            rssbytes += stat.rss_bytes();
            for cid in self.children.get(pid).unwrap() {
                let (ticks, bytes) = self.stat_collect(cid);
                cputicks += ticks;
                rssbytes += bytes;
            }
        }
        return (cputicks, rssbytes);
    }
}
