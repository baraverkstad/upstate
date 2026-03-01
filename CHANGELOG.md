# Changelog


## Unreleased

### Added
- Added `CHANGELOG.md` file


## v2.3 - 2025-12-04

### Added
- `--no-summary` option to omit the system summary
- `--no-services` option to omit services section
- `--sort=<key>` option to sort services
- `--limit=<n>` option to limit number of services
- `justfile` as an alternative to `make` for build commands

### Changed
- Updated Docker base image to Alpine Linux 3.23
- Updated Rust edition from 2021 to 2024
- Adjusted `--version` output format

### Removed
- `--summary` command-line option (use `--no-services` instead)
- Configuration and man page files from Docker image

### Fixed
- Panic on macOS due to reported memory sizes
- Various code style issues flagged by Clippy


## v2.2 - 2025-05-11

### Added
- Support for `upstate.conf.d` directory with part files

### Changed
- Updated Docker base image to Alpine Linux 3.21
- Updated dependencies: `colored` v3.0, `regex` v1.11, `sysinfo` v0.35, `procfs` v0.17

### Removed
- `upstate.sh` shell script from the install script
- Publishing of pre-built Docker images

### Fixed
- `make test` now properly checks all shell files


## v2.1 - 2024-08-05

### Added
- Support for prefixing process names with `+` and `*` (in addition to `-`)
- Check to avoid listing the same process ID multiple times
- Support for multiple service processes sharing the same name

### Changed
- Updated Docker base image to latest Alpine Linux
- Process RSS and time calculations now use `sysinfo` more directly

### Removed
- Pre-built Docker images (due to multi-arch build times)

### Fixed
- Load average formatting to always show 2 decimal places
- Various issues in `install.sh` script


## v2.0 - 2023-10-13

### Added
- Complete rewrite in Rust, replacing the original Bash shell script
- Binary downloads for most common Linux machines and Docker hosts
- GitHub Actions workflow for automatic builds and Docker image publishing
- Docker image for `linux/arm/v6` platform

### Changed
- Binaries are (almost) static, requiring only `libgcc`
- Migrated Docker image from Docker Hub to GitHub Container Registry
- Updated `install.sh` to prefer binary installation when available
- Adjusted output columns for process statistics

### Removed
- Runtime dependencies on `pstree`, `grep`, `bash`, and similar shell tools

### Fixed
- Trim PPID from `ps` output
- Corrected `grep` switch for Perl-compatible regular expressions


## v1.0 - 2022-09-01

### Added
- Initial release of upstate
- Shell script implementation monitoring Linux system and process metrics
- Dockerfile for running upstate from a Docker container
- `install.sh` script for remote installation via `curl`

