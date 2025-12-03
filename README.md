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
      --no-summary  Exclude machine status from output.
      --no-services Exclude services list from output.
      --limited     Include machine status and configured services.
      --complete    Include machine status and all services (default).
      --sort=<key>  Sort services by cpu, rss, or uptime.
      --limit=<n>   Limit the number of services shown.
      --json        Print the report in JSON output format.

    Returns:
      Non-zero if one or more configured services weren't found.

    Files:
      /etc/upstate.conf
```


## Installation

The easiest installation is to use the installer script:

```
    curl -L https://raw.githubusercontent.com/baraverkstad/upstate/main/install.sh | bash
```

As an alternative, the `./install.sh` script can also be run directly from an
unpacked download directory.

A third option is to manually copy the `upstate` binary, `man/man1/upstate.1` and
`etc/upstate.conf` to their desired locations on the server.

Finally, it is also possible to run as a Docker container with access to the host
machine processes and PID files:

```
    docker run --rm --tty --pid host \
        -v /etc/upstate.conf:/etc/upstate.conf:ro \
        -v /var/run:/var/run:ro \
        ghcr.io/baraverkstad/upstate:latest
```


## Configuration

The processes to check are configured in a single `upstate.conf` file or an
`upstate.conf.d` directory with partial config files. These files are located
based on the binary location or the current working dir. As an alternative
the `UPSTATE_CONF` environment variable may point to either a file or
directory with configuration.

The configuration files should contain one line per process. Comment or blank
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

Processes with a leading `+` character in the name may match multiple
processes (at least one).

Processes with a leading `*` character in the name are optional, but
may match multiple processes. These are omitted in `--limited` report
and always ignored if missing.

A pid file specified as `-` forces process lookup by either name or
command-line match. Process lookup is also made in a similar way if the pid
file didn't exist or didn't match a running process.


## See Also

* [df](http://manpages.ubuntu.com/manpages/man1/df.1.html)
* [pgrep](http://manpages.ubuntu.com/manpages/man1/pgrep.1.html)
* [ps](http://manpages.ubuntu.com/manpages/man1/ps.1.html)
* [pstree](http://manpages.ubuntu.com/manpages/man1/pstree.1.html)
* [proc](http://manpages.ubuntu.com/manpages/man5/proc.5.html)
