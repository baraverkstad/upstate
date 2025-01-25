use colored::Colorize;
use humansize::{format_size, FormatSizeOptions, BINARY};
use indoc::indoc;
use std::collections;
use std::fmt::Display;
use std::process;
use std::time;
use sysinfo::*;

mod conf;
mod fmt;
mod proc;

fn error<T: Display>(msg: T) {
    eprintln!("\n{}: {}", "ERROR".red(), msg);
}

fn warning<T: Display>(msg: T) {
    eprintln!("{}: {}", "WARNING".yellow(), msg);
}

fn usage() {
    eprint!(indoc! {r#"
        Prints a machine and service status report.

        Syntax: upstate [options]

        Options:
          --summary     Include only a short machine status.
          --limited     Include machine status and configured services.
          --complete    Include machine status and all services (default).
          --json        Output report in JSON format.

        Returns:
          Non-zero if one or more configured services were missing.

        Files:
          /etc/upstate.conf
    "#});
}

fn main() {
    let mut mode = 2;
    let mut fmt = fmt::Format::Text;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--summary" => mode = 0,
            "--limited" => mode = 1,
            "--complete" => mode = 2,
            "--json" => fmt = fmt::Format::json(),
            "--help" | "-h" | "-?" => {
                usage();
                process::exit(0);
            }
            "--version" => {
                eprintln!(
                    "Upstate \u{2219} {} ({}, rev. {})",
                    env!("VERSION"),
                    env!("DATE"),
                    env!("COMMIT")
                );
                eprintln!("# Server metrics for man & machine. See --help for details.");
                process::exit(0);
            }
            unknown => {
                usage();
                error(format!("invalid command-line argument: {}", unknown));
                process::exit(1);
            }
        }
    }
    let sys = System::new_all();
    fmt.json_open("", false, true);
    cpusummary(&sys, &mut fmt);
    memsummary(&sys, &mut fmt);
    storagesummary(&mut fmt);
    let mut ret = 0;
    if mode > 0 {
        let config = conf::Config::new().unwrap_or_else(|err| {
            warning(err);
            return conf::Config::empty();
        });
        ret = procsummary(&sys, &mut fmt, &config, mode > 1);
    }
    fmt.json_close(false);
    process::exit(ret);
}

fn elapsed(secs: u64) -> String {
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    if days > 0 {
        return format!("{} days", days);
    } else {
        return format!("{:02}:{:02}:{:02}", hours, mins % 60, secs % 60);
    }
}

fn cpusummary(sys: &System, fmt: &mut fmt::Format) {
    let cores = sys.physical_core_count().unwrap_or(1);
    let uptime = System::uptime();
    let loadavg = System::load_average();
    let load = format!("{:.2}, {:.2}, {:.2}", loadavg.one, loadavg.five, loadavg.fifteen);
    let procs = sys.processes().len();
    let detail = vec![
        format!("up {}", elapsed(uptime)),
        format!("{} processes", procs),
        format!("{} cores", cores),
    ];
    fmt.text_summary("loadavg:", &load, &detail.join(" \u{2219} "));
    fmt.json_key_val("cores", cores);
    fmt.json_key_val("uptime", uptime);
    fmt.json_key_val("loadavg", format!("[{}]", &load));
    fmt.json_key_val("processes", procs);
}

fn memsummary(sys: &System, fmt: &mut fmt::Format) {
    let sizefmt = FormatSizeOptions::from(BINARY).decimal_places(1);
    let total = sys.total_memory();
    let free = sys.free_memory();
    let freepct = 100_f64 * free as f64 / total as f64;
    let cache = sys.available_memory() - free;
    let rss = sys.used_memory();
    let swap = sys.used_swap();
    let mem = format!("{} ({:.1}%) free", format_size(free, sizefmt), freepct);
    let mut detail = vec![
        format!("{} rss", format_size(rss, sizefmt)),
        format!("{} cache", format_size(cache, sizefmt)),
        format!("{} total", format_size(total, sizefmt)),
    ];
    if swap > 0 {
        detail.insert(2, format!("{} swap", format_size(swap, sizefmt)));
    }
    fmt.text_summary("memory:", &mem, &detail.join(" \u{2219} "));
    fmt.json_open("memory", false, true);
    fmt.json_key_val("total", total);
    fmt.json_key_val("free", free);
    fmt.json_key_val("rss", rss);
    fmt.json_key_val("cache", cache);
    fmt.json_key_val("swap", swap);
    fmt.json_close(false);
}

fn storagesummary(fmt: &mut fmt::Format) {
    let sizefmt = FormatSizeOptions::from(BINARY).decimal_places(1);
    let mut devices = collections::HashSet::new();
    fmt.json_open("storage", true, true);
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        if let DiskKind::Unknown(_) = disk.kind() {
            continue;
        } else if devices.contains(disk.name()) {
            continue;
        } else {
            devices.insert(disk.name());
        }
        let total = disk.total_space();
        let avail = disk.available_space();
        let availpct = 100_f64 * avail as f64 / total as f64;
        let info = format!("{} ({:.1}%) free", format_size(avail, sizefmt), availpct);
        let detail = vec![
            format!("{} used", format_size(total - avail, sizefmt)),
            format!("{} total", format_size(total, sizefmt)),
            format!("on {}", disk.mount_point().display()),
        ];
        fmt.text_summary("storage:", &info, &detail.join(" \u{2219} "));
        fmt.json_open("", false, false);
        fmt.json_key_val("total", total);
        fmt.json_key_val("used", total - avail);
        fmt.json_key_val("free", avail);
        fmt.json_key_str("dev", disk.name().to_str().unwrap_or(""));
        fmt.json_key_str("mount", disk.mount_point().display());
        fmt.json_close(false);
    }
    fmt.json_close(true);
}

fn procsummary(sys: &System, fmt: &mut fmt::Format, conf: &conf::Config, all: bool) -> i32 {
    let now = time::SystemTime::now();
    let epoch = now.duration_since(time::UNIX_EPOCH).unwrap().as_secs();
    let procs = proc::ProcessMap::new(&sys);
    let mut found = vec![];
    let mut errors = 0;
    fmt.json_open("services", true, true);
    for (title, pid, err) in conf.all(&procs) {
        if pid <= 0 {
            let name = format!("{} [{}]", title, "?");
            fmt.text_proc_err(name, err.clone());
            fmt.json_open("", false, false);
            fmt.json_key_val("pid", 0);
            fmt.json_key_str("name", title);
            fmt.json_key_str("error", err);
            fmt.json_close(false);
            errors += 1;
        } else if !found.contains(&pid) {
            found.push(pid);
            let proc = sys.process(Pid::from_u32(pid)).unwrap();
            let uptime = epoch - proc.start_time();
            let (cputime, rssbytes) = procs.stat(&pid);
            procitem(fmt, pid, title, cputime, uptime, rssbytes, &err);
            if err.len() > 0 {
                fmt.text_proc_more("Warning:", err);
            }
        }
    }
    let mut services = procs.services();
    services.sort();
    for pid in services {
        if all && !found.contains(&pid) {
            let proc = sys.process(Pid::from_u32(pid)).unwrap();
            if !proc.exe().is_some() && proc.memory() == 0 {
                // Lets ignore kernel threads
                continue;
            }
            let uptime = epoch - proc.start_time();
            let (cputime, rssbytes) = procs.stat(&pid);
            let warn = "service not listed in config";
            procitem(fmt, pid, proc.name().to_str().unwrap(), cputime, uptime, rssbytes, warn);
        }
    }
    fmt.json_close(true);
    return errors;
}

fn procitem(fmt: &mut fmt::Format, pid: u32, name: &str, cpu: u64, up: u64, rss: u64, warn: &str) {
    let sizefmt = FormatSizeOptions::from(BINARY).decimal_places(1);
    let label = format!("{} [{}]", name, pid);
    let detail = vec![
        format!("cpu {}", elapsed(cpu)),
        format!("up {}", elapsed(up)),
        format!("{} rss", format_size(rss, sizefmt)),
    ];
    if warn.len() > 0 {
        fmt.text_proc_warn(label, detail.join(" \u{2219} "));
    } else {
        fmt.text_proc_ok(label, detail.join(" \u{2219} "));
    }
    fmt.json_open("", false, false);
    fmt.json_key_val("pid", pid);
    fmt.json_key_str("name", name);
    fmt.json_key_val("cputime", cpu);
    fmt.json_key_val("uptime", up);
    fmt.json_key_val("rss", rss);
    if warn.len() > 0 {
        fmt.json_key_str("warning", warn);
    }
    fmt.json_close(false);
}
