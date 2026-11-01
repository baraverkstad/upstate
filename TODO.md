# TODO

Keep Upstate small: collect an accurate, useful report quickly.

- [ ] Update outdated dependencies first. Use `make outdated` to review and
      `make test` to verify updates.
- [ ] Add OS name/version and kernel version using the existing `sysinfo`
      dependency.
- [ ] Improve automatic service names using local cgroup metadata, including
      container IDs. Follow the [service-name plan](PLAN-auto-service-names.md),
      omitting its optional Docker/Podman CLI lookup.

Docker container state, image details, and image/volume storage accounting are
out of scope; defer to dedicated tools.
