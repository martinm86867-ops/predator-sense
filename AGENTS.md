# AGENTS.MD: Using `witr` ("Why Is This Running?")

A practical reference guide for autonomous agents, automated workflows, and terminal operators on how to use `witr` to inspect, trace, and diagnose processes, network ports, containers, and file descriptors.

---

## 1. What is `witr`?

`witr` answers a fundamental operational question: **"Why is this running?"**

Unlike standard tools like `ps`, `top`, or `pgrep` which only report *that* a process exists, `witr` reconstructs the entire **causal chain**—identifying the supervisor, container engine, systemd unit, cron job, or parent shell that launched it.

Binary location on this system:
```bash
/usr/local/bin/witr
```

---

## 2. Core Agent Patterns & Commands

### A. Trace Process Lineage by PID (Deterministic Inspection)
When an agent or script identifies a problematic or high-resource PID:

```bash
# Clean, single-line lineage chain (ideal for script parsing / logging)
witr --pid <PID> --short --no-color

# Full process ancestry as an ASCII tree
witr --pid <PID> --tree --no-color
```

**Example Output:**
```text
systemd (pid 1) → systemd (pid 4220) → gnome-shell (pid 4741)
```

---

### B. Machine-Readable JSON Output (`--json`)
For autonomous agents that need to parse process state programmatically:

```bash
witr --pid <PID> --json
```

**Schema Overview:**
```json
{
  "Target": {
    "Type": "pid",
    "Value": "1"
  },
  "ResolvedTarget": "systemd",
  "Process": {
    "PID": 1,
    "PPID": 0,
    "Command": "systemd",
    "Cmdline": "/usr/lib/systemd/systemd ...",
    "User": "root",
    "CPUPercent": 0.03,
    "MemoryRSS": 17063936,
    "WorkingDir": "/",
    "Capabilities": ["CAP_CHOWN", "CAP_DAC_OVERRIDE"]
  },
  "Chain": [ ... ]
}
```

---

### C. Identify What is Listening on a Port (`--port`)
When binding a socket or debugging conflicting microservices (e.g. port 3000, 8080, 5432):

```bash
witr --port <PORT_NUMBER> --json
```

**Example:**
```bash
witr --port 3080 --short
# Traces exactly what process owns port 3080 and what parent started it
```

---

### D. Identify What Holds a File Lock (`--file`)
When apt, dpkg, or a local database is blocked by a file lock:

```bash
witr --file /var/lib/dpkg/lock-frontend --short
```

---

### E. Container-Aware Inspection (`--container`)
To trace the host process representing a running container:

```bash
witr --container <CONTAINER_NAME> --tree
```

---

### F. Security & Anomaly Warnings (`--warnings`)
Audit a process for suspicious environment variables, deleted executables running in RAM, or unexpected parentage:

```bash
witr <PROCESS_NAME> --warnings
```

---

## 3. Integration in Rust / Python / Shell

### Calling `witr` from Rust (e.g. `predator-sense`):
```rust
use std::process::Command;

pub fn trace_lineage(pid: u32) -> Option<String> {
    let output = Command::new("witr")
        .args(["--pid", &pid.to_string(), "--short", "--no-color"])
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}
```

### Calling `witr` from Python:
```python
import subprocess
import json

def get_process_lineage(pid: int) -> dict:
    result = subprocess.run(
        ["witr", "--pid", str(pid), "--json"],
        capture_output=True,
        text=True,
        check=True
    )
    return json.loads(result.stdout)
```

### Calling `witr` in Bash Scripts:
```bash
LINEAGE=$(witr --pid "$TARGET_PID" --short --no-color 2>/dev/null)
echo "[INFO] Process ancestry: $LINEAGE"
```

---

## 4. Best Practices for Agents

1. **Always pass `--no-color` in headless automation:** Prevents ANSI escape sequences from polluting regex matches and JSON outputs.
2. **Prefer `--pid` over process names:** Avoids ambiguity when multiple instances of a command (e.g. `python`, `node`, `sh`) exist.
3. **Use `--short` for concise logs and UI labels:** Generates human-readable breadcrumb chains (e.g. `init → systemd → service`).
4. **Combine with Process Termination:** Use `witr` to confirm that a process is not a critical parent/system daemon before dispatching `kill -15` or `kill -9`.
