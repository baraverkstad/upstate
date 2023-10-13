#!/usr/bin/env bash
# shellcheck disable=SC2064

# Global variables
PROGRAM="$0"

# Set caution flags
set -o nounset
set -o errtrace
set -o errexit
set -o pipefail

# Logs an error and exits (code 1)
die() {
    echo "ERROR:" "$@" >&2
    exit 1
}

print_os() {
    case $(uname -s) in
    "Linux") echo -n "linux" ;;
    *)       echo -n "" ;;
    esac
}

print_arch() {
    case $(uname -m) in
    "amd64"|"x86_64")  echo -n "amd64" ;;
    "arm64"|"aarch64") echo -n "arm64" ;;
    "armv6l")          echo -n "armv6" ;;
    "armv7l")          echo -n "armv7" ;;
    *)                 echo -n "" ;;
    esac
}

print_libc() {
    case "${OSTYPE:-}" in
    *-gnu)  echo -n "gnu" ;;
    *-musl) echo -n "musl" ;;
    *)      echo -n "" ;;
    esac
}

# Downloads a file from a URL
download_url() {
    local FILE="$1" URL="$2"
    echo "# Downloading ${URL}..."
    if command -v curl > /dev/null ; then
        curl --silent --location -o "${FILE}" "${URL}"
    elif command -v wget > /dev/null ; then
        wget --quiet -O "${FILE}" "${URL}"
    elif command -v aria2c > /dev/null ; then
        aria2c -o "${FILE}" "${URL}"
    elif command -v pget > /dev/null ; then
        pget -o "${FILE}" "${URL}"
    else
        die "couldn't locate curl, wget or similar command"
    fi
}

# Downloads and unpacks the source files
download_files() {
    if [[ $(basename "${PROGRAM}") == 'install.sh' ]] ; then
        cd "$(dirname "${PROGRAM}")"
    else
        local DIR URL OS ARCH LIBC
        DIR=$(mktemp --tmpdir --directory upstate-install-XXXXXXXX)
        trap "rm -rf ${DIR}" EXIT
        cd "${DIR}"
        if [[ -z "${VERSION:-}" ]] ; then
            URL="https://github.com/baraverkstad/upstate/archive/master.zip"
        else
            URL="https://github.com/baraverkstad/upstate/archive/v${VERSION}.zip"
        fi
        OS=$(print_os)
        ARCH=$(print_arch)
        LIBC=$(print_libc)
        if [[ -n "${OS:-}" && -n "${ARCH:-}" && -n "${LIBC:-}" ]] ; then
            URL="https://github.com/baraverkstad/upstate/releases/download/${VERSION:-latest}/upstate-${OS}-${ARCH}-${LIBC}.zip"
        fi
        download_url upstate.zip "${URL}"
        command -v unzip > /dev/null || die "couldn't locate unzip command"
        unzip -q -u -o upstate.zip
        cd upstate-*
    fi
}

# Installs the source files
install_files() {
    echo "Installing to /usr/local/bin/..."
    if [[ -f bin/upstate ]] ; then
        install -D bin/upstate /usr/local/bin/upstate
    else
        install -D bin/upstate.sh /usr/local/bin/upstate
    fi
    echo "Installing to /usr/local/share/man/..."
    install -D --mode=644 man/man1/upstate.1 /usr/local/share/man/man1/upstate.1
    gzip -f /usr/local/share/man/man1/upstate.1
    echo "Installing to /usr/local/share/upstate/..."
    install -D --mode=644 etc/upstate.conf /usr/local/share/upstate/upstate.conf
    if [[ ! -r /etc/upstate.conf ]] && [[ ! -r /usr/local/etc/upstate.conf ]] ; then
        install -D --mode=644 etc/upstate.conf /usr/local/etc/upstate.conf
        echo
        echo "An example /usr/local/etc/upstate.conf file has been installed."
        echo "Please edit to match your server configuration."
    fi
}

[[ $(whoami) == 'root' ]] || die "only root can run the installation"
download_files
install_files
echo
echo "Done! Upstate now installed!"
