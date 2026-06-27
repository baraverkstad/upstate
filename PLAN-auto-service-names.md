# Plan: Auto Service Names for Unconfigured Processes

## Context

`upstate --complete` lists unconfigured PPID=1 processes ("other services") using
`proc.name()` — the kernel comm name, truncated to 15 chars on Linux. The goal is to
show a meaningful identifier for each process by consulting Linux-specific sources.

Current code (`main.rs:282`):
```rust
name: proc.name().to_str().unwrap_or_default().to_string(),
```

sysinfo is lowest-common-denominator cross-platform — no cgroup or systemd knowledge.
We supplement with Linux-only logic reading `/proc/<pid>/cmdline` and `/proc/<pid>/cgroup`
— pure file I/O, no subprocess, no new dependencies.

---

## Display Format

Use slash-namespace prefix to indicate type. Plain name for systemd services (most
common case, no decoration needed):

```
■ sshd                              6d  0.1%  12MB   ← systemd service
■ nginx                             2d  0.2%   8MB   ← systemd service
■ docker/a3f9c2d1b5e7               1h  0.0%   4MB   ← Docker container (12-char ID)
■ podman/a3f9c2d1b5e7               3h  0.1%   6MB   ← Podman container (12-char ID)
■ k8s/a3f9c2d1b5e7                  5m  0.0%   2MB   ← Kubernetes container
■ vm/dev-sandbox                    1d  0.3%  64MB   ← systemd-nspawn / LXC machine
```

If container names are resolved via CLI (phase 2), IDs are replaced with names:
```
■ docker/nginx_web_1
■ podman/my-app
```

---

## Resolution Hierarchy

For each unconfigured process, try in order, return first match:

1. **Own cgroup** (`/proc/<pid>/cgroup`) — scan components for container scopes, VM scopes, then `.service` name
2. **First child's cgroup** — same container-scope scan on the first child PID (covers containerd shims, whose own cgroup path is `containerd.service` with no container ID)
3. **Fallback** — `proc.name()` comm (current behavior)

`ProcessMap` already holds `children: HashMap<u32, Vec<u32>>` built from sysinfo — no extra I/O or data structures needed.

---

## Step 1: Container Scope via Cgroup

Read `/proc/<pid>/cgroup`. Find the line starting with `0::`, take the path after it,
split by `/`, and scan components for container scope patterns.

Example:
```
0::/system.slice/docker-9cec00b45bcf.....scope
```

### Scope name patterns (match any path component)

| Component pattern | Label |
|---|---|
| `docker-<64hex>.scope` | `docker/<id[..12]>` |
| `libpod-<64hex>.scope` (not `libpod-conmon-`) | `podman/<id[..12]>` |
| `libpod-conmon-<64hex>.scope` | `podman/<id[..12]>` |
| `cri-containerd-<64hex>.scope` | `k8s/<id[..12]>` |
| `crio-<64hex>.scope` | `k8s/<id[..12]>` |
| `nerdctl-<64hex>.scope` | `ctr/<id[..12]>` |

Match: strip the known prefix, check remaining ≥ 64 hex chars before `.scope`.

If no container scope matched, the same cgroup path is also checked for VM and service patterns (steps 2 and 3 below). This is one file read, one pass.

---

## Step 2: VM / Machine via Cgroup

After container checks, look for `machine-*.scope` components under `machine.slice`.

Component pattern: `machine-<name>.scope`
Label: `vm/<name>`

Systemd encodes the machine name in the scope: `-` in the scope name represents `-`
in the machine name (machinectl names are `[a-zA-Z0-9_.-]`, no path separators, so
no encoding needed). Strip `machine-` prefix and `.scope` suffix.

Examples:
```
0::/machine.slice/machine-dev--sandbox.scope   → vm/dev--sandbox
0::/machine.slice/machine-ubuntu.scope          → vm/ubuntu
```

---

## Step 3: systemd Unit Name via Cgroup

If no container/VM pattern matched, scan path components **right-to-left** for the
first component ending in `.service`. Strip the suffix.

**Why right-to-left**: some services place workers in sub-cgroups, e.g.
`systemd-udevd.service/udev` — the `.service` component is not last.

Examples:
```
0::/system.slice/sshd.service                    → sshd
0::/system.slice/systemd-udevd.service/udev      → systemd-udevd
0::/system.slice/sshd.service/sshd-session-1     → sshd
```

Non-service cgroups (`.scope` user sessions, bare slices, init.scope) → no match,
fall through to step 5.

---

## Step 4: Child Cgroup (Shim Fallback)

If steps 1–3 on the process's own cgroup yielded nothing useful, try the same
container-scope patterns (step 1 only — not VM or service) on the first child PID
from `ProcessMap.children`. This covers containerd-shim processes, whose own cgroup
path is `0::/system.slice/containerd.service` (no container ID), but whose single
child (the container init) lands in `0::/system.slice/docker-<64hex>.scope`.

No extra I/O to find the child PID — `ProcessMap.children` is already populated.

---

## Step 5: Fallback

Return `comm` (current behavior — `proc.name().to_str().unwrap_or_default()`).

---

## Implementation

### `src/proc.rs` — new Linux-only method on `ProcessMap`

```rust
#[cfg(target_os = "linux")]
pub fn service_label(&self, pid: u32, comm: &str) -> String {
    label_from_cgroup(pid, false)
        .or_else(|| {
            self.children.get(&pid)
                .and_then(|c| c.first())
                .and_then(|child| label_from_cgroup(*child, true))
        })
        .unwrap_or_else(|| comm.to_string())
}

#[cfg(not(target_os = "linux"))]
pub fn service_label(&self, _pid: u32, comm: &str) -> String {
    comm.to_string()
}
```

#### `label_from_cgroup(pid, container_only: bool) -> Option<String>`

```
read /proc/<pid>/cgroup as string
find line starting with "0::", take path after it → else return None
split path by '/'
for each component (left-to-right):
  check container scope patterns → return docker/podman/k8s/ctr label
  if not container_only: check machine-*.scope → return vm/<name>
if not container_only:
  for each component (right-to-left):
    if ends with ".service" → return component stripped of ".service"
None
```

`container_only: true` is used for child cgroup lookups — we don't want a shim's
container-init child to match VM or service patterns.

### `src/main.rs:282`

Replace:
```rust
name: proc.name().to_str().unwrap_or_default().to_string(),
```
With:
```rust
name: procs.service_label(pid, proc.name().to_str().unwrap_or_default()),
```

---

## Edge Cases

| Scenario | Behavior |
|---|---|
| Kernel threads | Already filtered (`exe.is_none() && memory == 0`) before label resolution |
| Non-systemd Linux | `/proc/<pid>/cgroup` exists but path is `/` or bare slice → falls through to comm |
| Template units (`getty@tty1.service`) | Returned as-is: `getty@tty1` |
| Transient units | Returned as-is |
| User sessions (`init.scope`, `session-N.scope`) | No `.service` component → fall through to comm |
| macOS | `service_label` returns comm unchanged (`#[cfg(not(target_os = "linux"))]`) |
| `/proc/<pid>/cgroup` unreadable | `label_from_cgroup` returns None → falls through |
| `/proc/<pid>/cmdline` unreadable | `label_from_cmdline` returns None → falls through |

---

## Phase 2: Container Name Resolution via CLI (optional, separate PR)

Once a container ID is extracted, resolve it to a human-readable name by running
`docker ps` and `podman ps` **once at startup** — one subprocess per runtime, not
one per container.

```sh
docker ps --no-trunc --format '{{.ID}}\t{{.Names}}'
podman ps --no-trunc --format '{{.ID}}\t{{.Names}}'
```

Build `HashMap<String, String>` (full 64-char ID → name) for each. Fail silently
(not installed, daemon down, insufficient permissions → skip).

When rendering: if label is `docker/<id12>`, look up `id12` prefix in the map,
replace with `docker/<name>` if found.

Docker requires `docker` group membership (socket access). Podman rootless requires
no special permissions. Kubernetes containers (`k8s/`) are not visible to `docker ps`
— short ID remains the display.

---

## Files Modified

- `src/proc.rs` — add `ProcessMap::service_label` method and `label_from_cgroup` helper
- `src/main.rs` — replace `proc.name()...` at line 282 with `service_label(...)`

No new dependencies. No changes to `src/fmt.rs`, `src/conf.rs`.

---

## Verification

1. `make build test`
2. On a Linux systemd host: `upstate --complete` — unconfigured services show `sshd`,
   `cron`, etc. instead of truncated comm names
3. With Docker containers running: shim processes show `docker/<id12>`
4. With Podman containers running: conmon shows `podman/<id12>`
5. On macOS: behavior unchanged
6. On a non-systemd Linux: graceful fallback to process names
