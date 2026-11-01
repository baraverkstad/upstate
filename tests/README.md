# Report tests

Run `make test` for Clippy, formatting checks, deterministic report and process
tests, and CLI tests. CLI tests set `UPSTATE_CONF` to fixtures and disable
colors; they do not use the machine's service configuration.

Run a clean build and the same checks on Linux in Orb:

```sh
orb make clean build test
```

The Linux smoke test collects live CPU, memory, storage and service metrics. It
checks types and basic relationships rather than exact values, and matches the
running Upstate process to verify service collection.

`fixtures/report.json` and `fixtures/report.txt` describe a healthy service, an
unconfigured service and a missing required service. Synthetic process trees
cover command matching, PID files and recursive CPU/RSS aggregation.

Two regression tests are ignored until the report-output TODO is fixed: JSON
string escaping and JSON delimiters appearing in text reports. Reproduce them
with:

```sh
orb env VERSION=test DATE=test COMMIT=test cargo test -- --ignored
```

These tests currently fail. Remove their `ignore` attributes when fixing the
output bugs. Collection failures leaving partial JSON remain a separate TODO.
