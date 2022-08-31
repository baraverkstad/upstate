# Upstate

Prints a report containing machine and process metrics on Linux. Also checks
for the existence of a number of configured processes. A short output example:

```
    loadavg:  0.06 0.04 0.00             up 264 days ∙ 192 processes ∙ 1 cores
    memory:   64 MB (6.6%) free          491 MB rss ∙ 426 MB cache ∙ 981 MB total
    storage:  16.0 GB (66.6%) free       6.8 GB used ∙ 24.1 GB total ∙ on /
    ● system/cron [297]                  1 MB rss ∙ up 264 days ∙ cpu 00:01:38
    ● ntp [1402230]                      2 MB rss ∙ up 159 days ∙ cpu 00:34:48
    ● sshd [1975344]                     23 MB rss ∙ up 108 days ∙ cpu 00:00:00
```


## Usage

```
    Syntax: upstate [options]

    Options:
      --summary     Report only a short machine status summary.
      --limited     Report excludes optional/hidden processes.
      --complete    Report includes all processes (default).
      --json        Prints the report in JSON output format.

    Returns:
      Non-zero if one or more configured services weren't found.

    Files:
      /etc/upstate.conf
```


## Configuration

The processes to check are configured in `/etc/upstate.conf` or
`/usr/local/etc/upstate.conf` with one line per process. Comment or blank
lines are ignored. Each line contains the process or service name, pid file
and an optional command-line argument to match:

```
    cron            /var/run/crond.pid
    sshd            /var/run/sshd.pid
    local           - local-command-match
    -rsyslog        /var/run/rsyslogd.pid
```

Processes with a leading `-` character in the name are optional. These
are omitted in `--limited` report and always ignored if missing.

A pid file specified as `-` forces process lookup by either name or
command-line match. Process lookup is also made in similar way if the pid
file didn't exist or didn't match a running process.


## See Also

* [df](http://manpages.ubuntu.com/manpages/man1/df.1.html)
* [pgrep](http://manpages.ubuntu.com/manpages/man1/pgrep.1.html)
* [ps](http://manpages.ubuntu.com/manpages/man1/ps.1.html)
* [pstree](http://manpages.ubuntu.com/manpages/man1/pstree.1.html)
* [proc](http://manpages.ubuntu.com/manpages/man5/proc.5.html)
