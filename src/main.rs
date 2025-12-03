use colored::Colorize;
use humansize::{BINARY, FormatSizeOptions, format_size};
use indoc::indoc;
use std::collections;
use std::fmt::Display;
use std::process;
use std::time;
use sysinfo::*;

mod conf;
mod fmt;
mod proc;

#[derive(PartialEq)]
enum SortBy {
    Cpu,
    Rss,
    Uptime,
}

struct ProcItem {
    pid: u32,
    name: String,
    cpu: u64,
    rss: u64,
    uptime: u64,
    warn: bool,
    msg: String,
}

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
          --no-summary  Exclude machine status from output.
          --no-services Exclude services list from output.
          --limited     Include machine status and configured services.
          --complete    Include machine status and all services (default).
          --sort=<key>  Sort services by cpu, rss, or uptime.
          --limit=<n>   Limit the number of services shown.
          --json        Print the report in JSON output format.

        Returns:
          Non-zero if one or more configured services were missing.

        Files:
          /etc/upstate.conf
    "#});
}

fn main() {
    let mut summary = true;
    let mut mode = 2;
    let mut fmt = fmt::Format::Text;
    let mut sort = None;
    let mut limit = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--no-summary" => summary = false,
            "--no-services" => mode = 0,
            "--limited" => mode = 1,
            "--complete" => mode = 2,
            "--json" => fmt = fmt::Format::json(),
            "--help" | "-h" | "-?" => {
                usage();
                process::exit(0);
            }
            "--version" => {
                eprintln!("Upstate ({}, {}, @{})", env!("VERSION"), env!("DATE"), env!("COMMIT"));
                eprintln!("# Server metrics for man & machine. See --help for details.");
                process::exit(0);
            }
            s if s.starts_with("--sort=") => {
                sort = match s.trim_start_matches("--sort=") {
                    "cpu" => Some(SortBy::Cpu),
                    "rss" | "mem" => Some(SortBy::Rss),
                    "time" | "uptime" => Some(SortBy::Uptime),
                    _ => {
                        error(format!("invalid sort option: {}", s));
                        process::exit(1);
                    }
                };
            }
            s if s.starts_with("--limit=") => {
                limit = match s.trim_start_matches("--limit=").parse() {
                    Ok(n) => Some(n),
                    Err(_) => {
                        error(format!("invalid limit option: {}", s));
                        process::exit(1);
                    }
                };
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
    if summary {
        cpusummary(&sys, &mut fmt);
        memsummary(&sys, &mut fmt);
        storagesummary(&mut fmt);
    }
    let mut ret = 0;
    if mode > 0 {
        let config = conf::Config::new().unwrap_or_else(|err| {
            warning(err);
            conf::Config::empty()
        });
        ret = procsummary(&sys, &mut fmt, &config, mode > 1, sort, limit);
    }
    fmt.json_close(false);
    process::exit(ret);
}

fn elapsed(secs: u64) -> String {
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    if days > 0 {
        format!("{} days", days)
    } else {
        format!("{:02}:{:02}:{:02}", hours, mins % 60, secs % 60)
    }
}

fn cpusummary(sys: &System, fmt: &mut fmt::Format) {
    let cores = System::physical_core_count().unwrap_or(1);
    let uptime = System::uptime();
    let loadavg = System::load_average();
    let load = format!("{:.2}, {:.2}, {:.2}", loadavg.one, loadavg.five, loadavg.fifteen);
    let procs = sys.processes().len();
    let detail = [
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
    let cache = sys.available_memory().saturating_sub(free);
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
        let detail = [
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

fn procsummary(
    sys: &System,
    fmt: &mut fmt::Format,
    conf: &conf::Config,
    all: bool,
    sort: Option<SortBy>,
    limit: Option<usize>,
) -> i32 {
    let now = time::SystemTime::now();
    let epoch = now.duration_since(time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let procs = proc::ProcessMap::new(sys);
    let mut found = vec![];
    let mut items = vec![];
    let mut errors = 0;

    // Configured services
    for (title, pid, err) in conf.all(&procs) {
        if pid == 0 {
            items.push(ProcItem {
                pid: 0,
                name: title.to_string(),
                cpu: 0,
                rss: 0,
                uptime: 0,
                warn: false,
                msg: err,
            });
            errors += 1;
        } else if !found.contains(&pid) {
            found.push(pid);
            if let Some(proc) = sys.process(Pid::from_u32(pid)) {
                let uptime = epoch - proc.start_time();
                let (cputime, rssbytes) = procs.stat(&pid);
                items.push(ProcItem {
                    pid,
                    name: title.to_string(),
                    cpu: cputime,
                    rss: rssbytes,
                    uptime,
                    warn: !err.is_empty(),
                    msg: err,
                });
            }
        }
    }

    // Other services
    let mut services = procs.services();
    services.sort();
    for pid in services {
        if all
            && !found.contains(&pid)
            && let Some(proc) = sys.process(Pid::from_u32(pid))
        {
            if proc.exe().is_none() && proc.memory() == 0 {
                // Let's ignore kernel threads
                continue;
            }
            let uptime = epoch - proc.start_time();
            let (cputime, rssbytes) = procs.stat(&pid);
            items.push(ProcItem {
                pid,
                name: proc.name().to_str().unwrap_or_default().to_string(),
                cpu: cputime,
                rss: rssbytes,
                uptime,
                warn: true,
                msg: String::from(""),
            });
        }
    }

    // Sort & print
    if let Some(s) = sort {
        items.sort_by(|a, b| match s {
            SortBy::Cpu => b.cpu.cmp(&a.cpu),
            SortBy::Rss => b.rss.cmp(&a.rss),
            SortBy::Uptime => b.uptime.cmp(&a.uptime),
        });
    }
    if let Some(n) = limit
        && n < items.len()
    {
        items.truncate(n);
    }

    // Print
    fmt.json_open("services", true, true);
    for item in items {
        printitem(fmt, item);
    }
    fmt.json_close(true);
    errors
}

fn printitem(fmt: &mut fmt::Format, item: ProcItem) {
    let sizefmt = FormatSizeOptions::from(BINARY).decimal_places(1);
    let label = format!("{} [{}]", item.name, item.pid);
    let detail = [
        format!("cpu {}", elapsed(item.cpu)),
        format!("up {}", elapsed(item.uptime)),
        format!("{} rss", format_size(item.rss, sizefmt)),
    ];
    if item.pid == 0 {
        fmt.text_proc_err(label, item.msg.clone());
    } else if item.warn {
        fmt.text_proc_warn(label, detail.join(" \u{2219} "));
        if !item.msg.is_empty() {
            fmt.text_proc_more("Warning:", item.msg.clone());
        }
    } else {
        fmt.text_proc_ok(label, detail.join(" \u{2219} "));
    }
    fmt.json_open("", false, false);
    fmt.json_key_val("pid", item.pid);
    fmt.json_key_str("name", item.name);
    if item.pid == 0 {
        fmt.json_key_str("error", item.msg.clone());
    } else {
        fmt.json_key_val("cputime", item.cpu);
        fmt.json_key_val("uptime", item.uptime);
        fmt.json_key_val("rss", item.rss);
    }
    if item.warn {
        if item.msg.is_empty() {
            fmt.json_key_str("warning", "not listed in config");
        } else {
            fmt.json_key_str("warning", item.msg);
        }
    }
    fmt.json_close(false);
}
