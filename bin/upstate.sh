#!/usr/bin/env bash
#
# Prints a machine and process status report.
#
# Syntax: upstate [options]
#
# Options:
#   --summary     Report only a short machine status summary.
#   --limited     Report excludes optional/hidden processes.
#   --complete    Report includes all processes (default).
#   --json        Prints the report in JSON output format.
#
# Returns:
#   Non-zero if one or more configured services weren't found.
#
# Files:
#   /etc/upstate.conf
#

# Set caution flags
set -o nounset
set -o errtrace
set -o errexit
set -o pipefail

# Global variables
PROGRAM="$0"
ARGS=()
if [[ -t 0 ]] ; then
    COLOR_ERR="$(tput setaf 1; tput bold)"
    COLOR_WARN="$(tput setaf 3)"
    COLOR_OK="$(tput setaf 2)"
    COLOR_GREY="$(tput setaf 7)"
    COLOR_OFF="$(tput sgr0)"
else
    COLOR_ERR=""
    COLOR_WARN=""
    COLOR_OK=""
    COLOR_GREY=""
    COLOR_OFF=""
fi

# Logs an error and exits (code 1)
die() {
    error "$@"
    exit 1
}

# Logs an error to stderr
error() {
    echo "${COLOR_ERR}ERROR:${COLOR_OFF}" "$@" >&2
}

# Logs a warning to stderr
warn() {
    echo "${COLOR_WARN}WARNING:${COLOR_OFF}" "$@" >&2
}

# Prints command-line usage info and exits
usage() {
    local LINE
    tail -n +3 "${PROGRAM}" | while IFS= read -r LINE ; do
        if [[ ${LINE:0:1} = "#" ]] ; then
            echo "${LINE:2}" >&2
        else
            break
        fi
    done
    if [[ -n "$*" ]] ; then
        error "$*"
    fi
    exit 1
}

# Prints a label with a value
label() {
    printf "${COLOR_GREY}%-9s${COLOR_OFF} %-26s " "$1:" "$2"
}

# Prints a list of details
detail() {
    printf " \u2219 %s" "$@" | tail -c +6 | xargs -0 printf "${COLOR_GREY}%s${COLOR_OFF}"
}

# Prints the percentage quota of two values
percent() {
    echo "$@" | awk '{ printf("%.1f%", $1 * 100 / $2) }'
}

# Prints a kB value as MB
mb() {
    echo "$@" | awk '{ printf("%.0f MB", $1 / 1024) }'
}

# Prints a kB value as GB
gb() {
    echo "$@" | awk '{ printf("%.1f GB", $1 / 1048576) }'
}

# Prints seconds elapsed in readable format
elapsed() {
    local TOTAL=$1
    local SECS=$((TOTAL%60))
    local MINS=$((TOTAL/60%60))
    local HOURS=$((TOTAL/3600%24))
    local DAYS=$((TOTAL/86400))
    if [[ ${DAYS} -gt 0 ]] ; then
        printf '%s days' "${DAYS}"
    else
        printf '%02d:%02d:%02d' "${HOURS}" "${MINS}" "${SECS}"
    fi
}

# Finds first existing file
try() {
    local FILE
    for FILE in "$@" ; do
        if [[ -f "${FILE}" ]] ; then
            echo -n "${FILE}"
            return
        fi
    done
    false
}

# Prints CPU status summary
cpusummary() {
    local CORES=0 UPTIME=() ARR=() LOADAVG="" PROCS=0 ELAPSED=""
    CORES=$(grep -c processor /proc/cpuinfo)
    read -r -a UPTIME < /proc/uptime
    read -r -a ARR < /proc/loadavg
    LOADAVG=$(printf '%s, %s, %s' "${ARR[@]:0:3}")
    PROCS=$(awk -F / '{printf $2}' <<< "${ARR[3]}")
    ELAPSED=$(elapsed "${UPTIME[0]%.*}")
    label "loadavg" "${LOADAVG}" >&3
    detail "up ${ELAPSED}" "${PROCS} processes" "${CORES} cores" >&3
    echo >&3
    printf '  "cores": %s,\n' "${CORES}" >&4
    printf '  "uptime": %s,\n' "${UPTIME[0]%.*}" >&4
    printf '  "loadavg": [%s],\n' "${LOADAVG}" >&4
    printf '  "processes": %s,\n' "${PROCS}" >&4
}

# Prints memory status summary
memsummary() {
    local TOTAL=0 FREE=0 RSS=0 CACHE=0 SWAP=0 LABEL NUM
    while read -r LABEL NUM _ ; do
        case "${LABEL}" in
        "MemTotal:") TOTAL=$((TOTAL+"${NUM:-0}")) ;;
        "MemFree:") FREE=$((FREE+"${NUM:-0}")) ;;
        "Buffers:") CACHE=$((CACHE+"${NUM:-0}")) ;;
        "Cached:") CACHE=$((CACHE+"${NUM:-0}")) ;;
        "SwapTotal:") SWAP=$((SWAP+"${NUM:-0}")) ;;
        "SwapFree:") SWAP=$((SWAP-"${NUM:-0}")) ;;
        esac
    done < /proc/meminfo
    RSS=$((TOTAL-FREE-CACHE))
    local FREE_MB FREE_PCT RSS_MB CACHE_MB TOTAL_MB
    FREE_MB=$(mb "${FREE}")
    FREE_PCT=$(percent "${FREE}" "${TOTAL}")
    RSS_MB=$(mb "${RSS}")
    CACHE_MB=$(mb "${CACHE}")
    TOTAL_MB=$(mb "${TOTAL}")
    label "memory" "${FREE_MB} (${FREE_PCT}) free" >&3
    detail "${RSS_MB} rss" "${CACHE_MB} cache" "${TOTAL_MB} total" >&3
    echo >&3
    printf '  "memory": {"total": %s, "free": %s, "rss": %s, "cache": %s, "swap": %s},\n' \
        "${TOTAL}" "${FREE}" "${RSS}" "${CACHE}" "${SWAP}" >&4
}

# Prints storage status summary
storagesummary() {
    local VOLUMES DEVICE TOTAL_KB USED_KB FREE_KB MOUNT
    printf '  "storage": [\n' >&4
    VOLUMES=$(df -k | tail -n +2 | grep -P '^/dev/')
    if [[ -r /.dockerenv ]] || grep -q -P 'docker|lxc' /proc/$$/cgroup ; then
        VOLUMES=$(df -k / | tail -n +2)
    fi
    while read -r DEVICE TOTAL_KB USED_KB FREE_KB _ MOUNT ; do
        local FREE_GB FREE_PCT USED_GB TOTAL_GB
        FREE_GB=$(gb "${FREE_KB}")
        FREE_PCT=$(percent "${FREE_KB}" "${TOTAL_KB}")
        USED_GB=$(gb "${USED_KB}")
        TOTAL_GB=$(gb "${TOTAL_KB}")
        label "storage" "${FREE_GB} (${FREE_PCT}) free" >&3
        detail "${USED_GB} used" "${TOTAL_GB} total" "on ${MOUNT}" >&3
        echo >&3
        printf '    {"total": %s, "used": %s, "free": %s, "dev": "%s", "mount": "%s"},\n' \
            "${TOTAL_KB}" "${USED_KB}" "${FREE_KB}" "${DEVICE}" "${MOUNT}" >&4
    done <<< "${VOLUMES}"
    printf '    null\n' >&4
    printf '  ],\n' >&4
}

# Locates a service PID by PID file or name match
servicepid() {
    local NAME=$1 PIDFILE=${2:--} MATCH=${3:-${NAME}} PID
    if [[ -r "${PIDFILE}" ]] && PID=$(($(<"${PIDFILE}"))) && [[ -d "/proc/${PID}" ]] ; then
        echo -n "${PID}"
    elif PID=$(pgrep -P 1 -o "${MATCH}") ; then
        echo -n "${PID}"
    elif PID=$(pgrep -P 1 -o -f "${MATCH}") ; then
        echo -n "${PID}"
    elif PID=$(pgrep -o -f "${MATCH}" | xargs ps -o 'ppid=') ; then
        echo -n "${PID}"
    fi
}

# Calculates service memory, swap, etc (incl. descendants)
servicestats() {
    local PID=$1 PIDS=() RSS=0 SWAP=0 ARR VAL UPTIMES
    read -r -a PIDS < <(pstree -p "${PID}" | grep -o -P '[^}]\(\d+\)' | grep -o -P '\d+' | xargs)
    while read -r VAL ; do
        RSS=$((RSS+VAL))
    done < <(ps -o 'rss=' "${PIDS[@]}")
    for VAL in "${PIDS[@]}" ; do
        read -r -a ARR < <(grep VmSwap "/proc/${VAL}/status" 2>/dev/null || echo 0 0)
        SWAP=$((SWAP+ARR[1]))
    done
    UPTIMES=$(ps -o etimes=,times= "${PID}" | xargs)
    echo "${PID} ${RSS} ${SWAP} ${UPTIMES}"
}

# Prints a service status summary
serviceinfo() {
    local PID=${1:-} NAME=${2:-} PIDFILE=${3:--} STATS=() ERROR="" WARNING=""
    if [[ -z "${PID}" ]] ; then
        ERROR="service not running"
    elif [[ "${PIDFILE}" != "-" ]] && [[ ! -r "${PIDFILE}" ]] ; then
        ERROR="no PID file ${PIDFILE}, pid ${PID} found"
    elif [[ "${PIDFILE}" != "-" ]] && [[ "${PID}" != $(($(<"${PIDFILE}"))) ]] ; then
        ERROR="invalid PID file ${PIDFILE}, pid ${PID} found"
    fi
    printf '    {' >&4
    if [[ -z "${ERROR}" ]] ; then
        read -r -a STATS < <(servicestats "${PID}")
        if [[ -n "${NAME}" ]] ; then
            printf "${COLOR_OK}\u25CF${COLOR_OFF} %-34s " "${NAME} [${PID}]" >&3
        else
            NAME=$(ps -o 'comm=' "${PID}" | awk '{print $1}')
            printf "${COLOR_WARN}\u25A0${COLOR_OFF} %-34s " "${NAME} [${PID}]" >&3
            WARNING="service not listed in config"
        fi
        local RSS_MB UPTIME CPUTIME
        RSS_MB=$(mb "${STATS[1]}")
        UPTIME=$(elapsed "${STATS[3]}")
        CPUTIME=$(elapsed "${STATS[4]}")
        detail "${RSS_MB} rss" "up ${UPTIME}" "cpu ${CPUTIME}" >&3
        printf '"pid": %s, "name": "%s", "rss": %s, "swap": %s, "uptime": %s, "cputime": %s' \
            "${PID}" "${NAME}" "${STATS[1]}" "${STATS[2]}" "${STATS[3]}" "${STATS[4]}" >&4
        if [[ -n "${WARNING}" ]] ; then
            printf ', "warning": "%s"' "${WARNING}" >&4
        fi
    else
        printf "${COLOR_ERR}\u25A0 %-34s%s${COLOR_OFF} " "${NAME} [${PID:-?}]" >&3
        echo -n "${ERROR}" >&3
        printf '"pid": %s, "name": "%s", "error": "%s"' "${PID:-0}" "${NAME}" "${ERROR}" >&4
    fi
    echo >&3
    printf '},\n' >&4
    [[ -z "${ERROR}" ]]
    return $?
}

# Prints a service/process status summary
servicesummary() {
    local ERRORS=0 FOUND=() NAME PID PIDFILE MATCH
    printf '  "services": [\n' >&4
    while read -r NAME PIDFILE MATCH ; do
        { [[ "${NAME}" != "" ]] && [[ "${NAME:0:1}" != "#" ]] ; } || continue
        local OPTIONAL=false
        if [[ "${NAME:0:1}" == "-" ]] ; then
            NAME="${NAME:1}"
            OPTIONAL=true
        fi
        PID=$(servicepid "${NAME}" "${PIDFILE}" "${MATCH}")
        if "${OPTIONAL}" && [[ -z "${PID}" ]] ; then
            true
        elif "${OPTIONAL}" && [[ " ${ARGS[*]} " == *" limited "* ]] ; then
            true
        elif ! serviceinfo "${PID}" "${NAME}" "${PIDFILE}" ; then
            ERRORS=$((ERRORS+1))
        fi
        if [[ -n "${PID}" ]] ; then
            FOUND+=("${PID}")
        fi
    done
    if [[ " ${ARGS[*]} " != *" limited "* ]] ; then
        for PID in $(pgrep -P 1) ; do
            if [[ " ${FOUND[*]} " != *" ${PID} "* ]] ; then
                serviceinfo "${PID}"
            fi
        done
    fi
    printf '    null\n' >&4
    printf '  ]\n' >&4
    return "${ERRORS}"
}

# Parse command-line arguments
parseargs() {
    while [[ $# -gt 0 ]] ; do
        case "$1" in
        "--summary")
            ARGS+=("summary")
            shift
            ;;
        "--limited")
            ARGS+=("limited")
            shift
            ;;
        "--complete")
            ARGS+=("complete")
            shift
            ;;
        "--json")
            ARGS+=("json")
            shift
            ;;
        "-?"|"-h"|"--help")
            usage
            ;;
        *)
            usage "invalid command-line argument: $1"
            ;;
        esac
    done
}

# Program start
main() {
    local DIR CONFIG RETVAL=0
    DIR=$(dirname "$0")
    CONFIG=$(try /etc/upstate.conf /usr/local/etc/upstate.conf "${DIR}/../etc/upstate.conf" "${DIR}/upstate.conf")
    if [[ -f "${CONFIG}" ]] ; then
        exec < "${CONFIG}"
    else
        warn "no upstate.conf file found"
        exec < /dev/null
    fi
    if [[ " ${ARGS[*]} " == *" json "* ]] ; then
        exec 3>/dev/null
        exec 4>&1
    else
        exec 3>&1
        exec 4>/dev/null
    fi
    echo "{" >&4
    cpusummary
    memsummary
    storagesummary
    if [[ ! " ${ARGS[*]} " == *" summary "* ]] ; then
        if servicesummary ; then
            RETVAL=0
        else
            RETVAL=$?
        fi
    fi
    printf '}\n' >&4
    return "${RETVAL}"
}

# Parse command-line and launch
parseargs "$@"
main
