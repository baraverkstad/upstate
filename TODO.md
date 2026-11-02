# TODO

Keep Upstate small: collect an accurate, useful report quickly.

- [x] Update outdated dependencies and verify with `make build test`.
- [x] Add a small report test harness: text/JSON fixtures, escaping, service
      matching and aggregation, CLI flags and exit codes; Linux smoke checks in
      Orb.
- [x] Fix report output: escape JSON strings, keep JSON delimiters out of text,
      and avoid partial JSON when collection fails.
- [x] Extend `--json` with hostname, OS name/version, kernel version,
      architecture, and collection timestamp.
- [x] Report available and used machine memory with accurate field names in text
      and JSON output.
- [ ] Investigate [macOS available memory](#macos-available-memory) and the
      `sysinfo` calculation; report findings upstream if appropriate.
- [ ] Improve automatic service names using local cgroup metadata, including
      container IDs. Follow the [service-name plan](PLAN-auto-service-names.md),
      omitting its optional Docker/Podman CLI lookup. Include cgroup memory
      budgets/headroom in this larger work; define whose cgroup is reported and
      keep host memory totals distinct.

Docker container state, image details, and image/volume storage accounting are
out of scope; defer to dedicated tools.

## macOS Available Memory

`sysinfo` 0.40.1 calculates available memory as active + inactive + free pages.
This follows an Apple memory-pressure quantity, but includes active pages and
may overstate the memory available to a new workload. Its separately calculated
used memory can overlap with available memory, making percentage alerts
difficult to interpret. Linux instead reports the kernel's `MemAvailable`
estimate.

Other collectors, including gopsutil, use free + inactive pages on macOS. A more
conservative candidate is free + purgeable pages + excess secluded pages,
omitting file-backed pages. Apple describes secluded memory as a special pool
for camera startup; it is unclear whether its excess is exposed on macOS or
whether the available counters overlap. `free_count` already includes
speculative pages. Apple's related reclaimable-memory formula applies to
non-macOS systems and also includes file-backed pages.

Compare these estimates against memory pressure under different workloads before
choosing a formula or proposing a change to `sysinfo`.

- [sysinfo macOS calculation change](https://github.com/GuillaumeGomez/sysinfo/pull/1632)
- [Apple memory-status accounting](https://github.com/apple-oss-distributions/xnu/blob/main/doc/vm/memorystatus.md)
- [gopsutil macOS memory calculation](https://github.com/shirou/gopsutil/blob/master/mem/mem_darwin.go)
