# Changelog

_Lists all notable user-visible changes to this project._


## Unreleased

- Added support for TOML config files (`upstate.toml`, `upstate.toml.d/`)
- Changed `--json` output to minified format for .jsonl compatibility

## v2.3 - 2025-12-04

- Added `--no-summary` option to omit the system summary
- Added `--no-services` option to omit services section
- Added `--sort=<key>` option to sort services
- Added `--limit=<n>` option to limit number of services
- Changed `--version` output format
- Removed `--summary` command-line option (use `--no-services` instead)
- Removed configuration and man page files from Docker image
- Fixed panic on macOS due to reported memory sizes

## v2.2 - 2025-05-11

- Added support for `upstate.conf.d` directory with part files
- Removed `upstate.sh` shell script from the install script

## v2.1 - 2024-08-05

- Added support for prefixing process names with `+` and `*`
- Added check to avoid listing the same process ID multiple times
- Added support for multiple service processes sharing the same name
- Changed process RSS and time calculations now use `sysinfo` more directly
- Removed some Docker image platforms (due to multi-arch build times)
- Fixed load average formatting to always show 2 decimal places
- Fixed various issues in `install.sh` script

## v2.0 - 2023-10-13

- Added binary downloads for most common Linux machines and Docker hosts
- Added Docker image for `linux/arm/v6` platform
- Changed implementation to Rust, replacing the Bash shell script
- Changed binaries to (almost) static, requiring only `libgcc`
- Changed Docker image from Docker Hub to GitHub Container Registry
- Changed `install.sh` to prefer binary installation when available
- Changed output columns for process statistics
- Removed runtime dependencies on `pstree`, `grep`, `bash`, and similar
- Fixed PPID trimming in `ps` output
- Fixed `grep` switch for Perl-compatible regular expressions

## v1.0 - 2022-09-01

- Initial release of upstate
- Added shell script implementation monitoring Linux system and processes
- Added Dockerfile for running upstate from a Docker container
- Added `install.sh` script for remote installation via `curl`
