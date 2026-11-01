# TODO

Keep Upstate small: collect an accurate, useful report quickly.

- [x] Update outdated dependencies and verify with `make build test`.
- [x] Add a small report test harness: text/JSON fixtures, escaping, service
      matching and aggregation, CLI flags and exit codes; Linux smoke checks in
      Orb.
- [ ] Fix report output: escape JSON strings, keep JSON delimiters out of text,
      and avoid partial JSON when collection fails.
- [ ] Extend `--json` with hostname, OS name/version, kernel version,
      architecture, and collection timestamp.
- [ ] Add logical CPU count, available memory, and total/free swap to `--json`.
      Current fields already include physical cores (`cores`), free memory, and
      used swap (`memory.swap`).
- [ ] Clarify memory field names: machine `rss` currently means used memory.
- [ ] Improve automatic service names using local cgroup metadata, including
      container IDs. Follow the [service-name plan](PLAN-auto-service-names.md),
      omitting its optional Docker/Podman CLI lookup. Include cgroup memory
      budgets/headroom in this larger work; define whose cgroup is reported and
      keep host memory totals distinct.

Docker container state, image details, and image/volume storage accounting are
out of scope; defer to dedicated tools.
