# AGENTS.md

_CLI for system metrics and service checks, with text and JSON output._

## File Reference

- [README.md] — usage, installation and service configuration
- [TODO.md] — planned work and scope
- [tests/README.md] — test fixtures, CLI checks and known regressions

## Command Reference

```
make                         # list available make targets
make build test              # build for MacOS testing
orb make clean build test    # build for OrbStack testing on Linux
```

## Design Notes

- Every change should leave the code better than you found it. If the answer
  isn't an unconditional yes, you're not done.
- Keep Upstate small: collect an accurate, useful report quickly
