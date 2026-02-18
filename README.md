<div align="center">

# 🛡️ ClawTower

**OS-level runtime security for AI agents — any agent, any framework**

[![Build](https://img.shields.io/github/actions/workflow/status/ClawTower/ClawTower/ci.yml?branch=main&style=flat-square)](https://github.com/ClawTower/ClawTower/actions)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Release](https://img.shields.io/github/v/release/ClawTower/ClawTower?style=flat-square)](https://github.com/ClawTower/ClawTower/releases)

</div>

---

AI agents now run shell commands, edit files, call APIs, and sometimes use `sudo` on real infrastructure. Once they're executing, visibility drops fast — prompt guardrails and container boundaries don't tell you what's actually happening at the OS level.

ClawTower monitors at the kernel boundary (auditd, inotify, network policy) and is designed so the agent being watched **can't turn it off**. The binary is immutable (`chattr +i`), the admin key is Argon2-hashed and never stored, and every tamper attempt is logged and alerted on. This is the **"swallowed key" pattern**: the thing being watched can't fire the security guard.

> **Early-stage project.** ClawTower works and is battle-tested against its own adversarial pentest suite, but it hasn't been widely deployed yet. Expect rough edges. [Testers and contributors welcome.](CONTRIBUTING.md)

## Quick Start

### One-line Install

```bash
curl -sSL https://raw.githubusercontent.com/ClawTower/ClawTower/main/scripts/oneshot-install.sh | sudo bash
```

### Build from Source

```bash
git clone https://github.com/ClawTower/ClawTower.git
cd ClawTower
cargo build --release

# Install binaries
sudo install -m 755 target/release/clawtower /usr/local/bin/clawtower
sudo install -m 755 target/release/clawsudo /usr/local/bin/clawsudo

# Make immutable (the "swallowed key")
sudo chattr +i /usr/local/bin/clawtower
```

### Initial Setup

```bash
# 1. Create config directory and copy config
sudo mkdir -p /etc/clawtower
sudo cp config.toml /etc/clawtower/config.toml

# Customize via drop-in overrides (survives updates)
sudo mkdir -p /etc/clawtower/config.d
sudo nano /etc/clawtower/config.d/my-overrides.toml    # watched_users, slack webhook, etc.

# 2. Run the setup script (installs binary, creates dirs, sets up systemd)
sudo scripts/setup.sh

# 3. Admin key is auto-generated on first run — save it!
#    It is displayed ONCE in journalctl output and stored only as an Argon2 hash
```

> **Note:** Don't edit `config.toml` directly — it gets replaced on updates. Put your customizations in `/etc/clawtower/config.d/*.toml` drop-in files instead.

**Recommended hardening** (run after setup):

```bash
sudo scripts/setup-auditd.sh        # Syscall monitoring — highly recommended
sudo scripts/setup-sudoers-deny.sh  # Block agent from stopping ClawTower
sudo scripts/setup-slack.sh         # Slack alerts
```

**Optional integrations** (add as needed):

```bash
sudo scripts/setup-iptables.sh      # Network alert logging
sudo scripts/setup-apparmor.sh      # AppArmor confinement
sudo scripts/build-preload.sh       # Build LD_PRELOAD guard library
sudo scripts/enable-preload.sh      # Activate LD_PRELOAD guard
sudo scripts/setup-falco.sh         # Falco eBPF monitoring
sudo scripts/setup-samhain.sh       # Samhain file integrity
```

> 📖 **Full installation guide:** See [`docs/INSTALL.md`](docs/INSTALL.md) for prerequisites, hardening details, and recovery procedures.

### Available Scripts

| Script | Purpose |
|--------|---------|
| `scripts/setup.sh` | Full installation (binary, dirs, systemd service) |
| `scripts/install.sh` | Apply tamper-proof hardening (chattr +i, sudoers deny) |
| `scripts/configure.sh` | Interactive config wizard |
| `scripts/uninstall.sh` | Reverse hardening + remove (requires admin key) |
| `scripts/setup-auditd.sh` | Install auditd rules |
| `scripts/setup-audit-rules.sh` | Configure specific audit watch rules |
| `scripts/setup-iptables.sh` | Configure iptables logging |
| `scripts/setup-apparmor.sh` | Load AppArmor profiles |
| `scripts/setup-falco.sh` | Install/configure Falco |
| `scripts/setup-samhain.sh` | Install/configure Samhain |
| `scripts/setup-slack.sh` | Configure Slack webhooks |
| `scripts/setup-sudoers-deny.sh` | Sudoers deny rules for agent |
| `scripts/build-preload.sh` | Compile libclawtower.so |
| `scripts/enable-preload.sh` | Activate LD_PRELOAD guard |
| `scripts/sync-barnacle.sh` | Update BarnacleDefense pattern databases |
| `scripts/oneshot-install.sh` | Single-command install from GitHub |

---

## Agent-Agnostic

ClawTower works at the OS level — it doesn't hook into your agent's code or require SDK integration. If an agent runs on Linux under a user account, ClawTower can monitor that UID:

- OpenClaw, Nanobot, SuperAGI, Goose, memU
- Claude Code, Codex CLI, Aider, Continue
- Devin, SWE-agent, and other autonomous coding agents
- Custom agents built on LangChain, CrewAI, AutoGen, or raw API calls
- Any process running under a monitored user account

Point it at the UID your agent runs as and it starts watching.

## Where ClawTower Fits

ClawTower is **not** a replacement for container isolation. It's the other half of the stack.

If tools like NanoClaw are your prevention layer (sandbox the agent so it can't cause damage), ClawTower is your detection-and-forensics layer (watch what the agent actually does, alert, and provide a tamper-evident trail). **Contain what you can, monitor what you must.** You want both.

ClawTower operates at the runtime layer — the part that catches what static scanning and sandboxing can't:

- A skill passes VirusTotal but exfiltrates data through allowed network paths at runtime
- An agent's behavior changes after a context window is poisoned
- A legitimate tool (`curl`, `scp`) is used for unauthorized data transfer
- Someone tampers with the agent's identity or configuration files

## What It Monitors

- **Behavioral detection** — syscall-level classification of exfiltration, privilege escalation, persistence, reconnaissance, side-channel attacks, and container escapes via auditd. Distinguishes agent vs. human actors.
- **File integrity** — inotify-based sentinel with protected (quarantine + restore) and watched (diff + alert) policies. Content scanning on every change.
- **Cognitive file protection** — SHA-256 baselines for AI identity files (`SOUL.md`, `AGENTS.md`, etc.). Any modification is a CRITICAL alert.
- **Pattern engine** — regex databases for prompt injection, dangerous commands, privacy violations, and supply-chain IOCs. Compiled at startup, applied in real-time.
- **30+ security scanners** — periodic checks covering firewall, auditd config, SSH hardening, Docker, SUID binaries, open ports, crontabs, and more.
- **Hash-chained audit trail** — SHA-256 chain where each entry includes the previous hash. Retroactive edits are mathematically detectable.
- **`clawsudo`** — sudo proxy/gatekeeper. Every privileged command goes through YAML policy evaluation first. Denied = exit code 77 + alert.
- **Network policy** — allowlist/blocklist for outbound connections. Scans commands for embedded URLs.
- **API key vault proxy** — maps virtual keys to real ones so the agent never sees actual credentials. Built-in DLP scanning.
- **Terminal dashboard** — Ratatui TUI with tabbed views for alerts, network, FIM, system status, and config editing.
- **Slack alerts** — webhook notifications with severity filtering, backup webhook failover, and periodic heartbeats.
- **Auto-updater** — Ed25519-verified binary updates with the `chattr -i` → replace → `chattr +i` dance.
- **Log tamper detection** — monitors audit logs for truncation, deletion, and inode replacement.
- **Admin key system** — Argon2-hashed, generated once, never stored. Rate limited. Unix socket for authenticated runtime commands.

## Configuration

ClawTower uses a TOML config file (default: `/etc/clawtower/config.toml`). Key sections:

```toml
[general]
watched_users = ["1000"]        # Numeric UIDs to monitor (not usernames! find with: id -u <agent-user>)
min_alert_level = "info"        # info | warning | critical
log_file = "/var/log/clawtower/clawtower.log"

[slack]
webhook_url = "https://hooks.slack.com/services/..."
backup_webhook_url = ""         # Failover webhook
channel = "#security"
min_slack_level = "warning"     # Only send warnings+ to Slack
heartbeat_interval = 3600       # Health ping every hour (0 = off)

[auditd]
enabled = true
log_path = "/var/log/audit/audit.log"

[network]
enabled = true
source = "auto"                 # auto | journald | file
log_prefix = "CLAWTOWER_NET"
allowlisted_cidrs = ["192.168.0.0/16", "10.0.0.0/8"]

[sentinel]
enabled = true
quarantine_dir = "/etc/clawtower/quarantine"
shadow_dir = "/etc/clawtower/sentinel-shadow"
scan_content = true
debounce_ms = 200

[[sentinel.watch_paths]]
path = "/home/agent/.workspace/SOUL.md"
patterns = ["*"]
policy = "protected"            # protected = restore + alert; watched = diff + alert

[barnacle]
enabled = false
vendor_dir = "./vendor/barnacle/barnacle/skill/configs"

[scans]
interval = 300                  # Seconds between scan sweeps

[auto_update]
enabled = true
interval = 300                  # Check GitHub every 5 minutes

[proxy]
enabled = false
bind = "127.0.0.1"
port = 18790

[api]
enabled = false
bind = "0.0.0.0"
port = 18791

[policy]
enabled = true
dir = "./policies"              # YAML policy rules for clawsudo

[netpolicy]
enabled = false
mode = "blocklist"              # allowlist | blocklist
blocked_hosts = ["evil.com"]

[ssh]
enabled = true                  # Monitor SSH login events via journald
```

> 📖 **Full configuration reference:** See [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) for every field, type, default value, and TOML example.

## Usage

```bash
# Start with Terminal UI (default)
clawtower

# Run headless (servers, background monitoring)
clawtower run --headless

# One-shot security scan and exit
clawtower scan

# Show service status
clawtower status

# Interactive configuration wizard
clawtower configure

# Self-update to latest release
clawtower update

# Check for updates without installing
clawtower update --check

# Verify audit chain integrity
clawtower verify-audit

# Update BarnacleDefense pattern databases
clawtower sync

# Apply tamper-proof hardening
clawtower harden

# Tail service logs
clawtower logs

# Uninstall (requires admin key)
clawtower uninstall

# Admin key is auto-generated on first run and printed once — save it!
# It is stored only as an Argon2 hash at /etc/clawtower/admin.key.hash

# Use clawsudo instead of sudo for AI agents
clawsudo apt-get update
```

## Architecture Overview

```
┌────────────────────────────────────────────────────────────┐
│                      ClawTower Core                        │
│                                                            │
│  ┌───────────────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Auditd Watcher   │  │ Sentinel │  │ Journald │ Sources│
│  │ ┌───────────────┐ │  │ (inotify)│  │  Tailer  │        │
│  │ │Behavior Engine│ │  └────┬─────┘  └────┬─────┘        │
│  │ │BarnacleDefense│ │       │              │              │
│  │ │Policy Engine  │ │  ┌────┴────┐   ┌────┴────┐         │
│  │ └───────────────┘ │  │ Scanner │   │Firewall │         │
│  └────────┬──────────┘  │  Loop   │   │ Monitor │         │
│           │             └────┬────┘   └────┬────┘         │
│           ▼                  ▼              ▼              │
│  ┌──────────────────────────────────────────────────┐     │
│  │            raw_tx Channel (mpsc, cap=1000)       │     │
│  └──────────────────────┬───────────────────────────┘     │
│                         ▼                                  │
│  ┌──────────────────────────────────────────────────┐     │
│  │             Alert Aggregator                      │     │
│  │       (fuzzy dedup · rate-limit · suppress)       │     │
│  └──────┬────────────┬────────────┬─────────────────┘     │
│         ▼            ▼            ▼                        │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐                │
│  │  Slack   │  │   TUI    │  │Audit Chain│  Outputs       │
│  │ Notifier │  │Dashboard │  │  (log)    │                │
│  └──────────┘  └──────────┘  └───────────┘                │
│                                                            │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐                │
│  │  REST    │  │  Proxy   │  │  Admin    │  Services      │
│  │  API     │  │  (DLP)   │  │  Socket   │                │
│  └──────────┘  └──────────┘  └───────────┘                │
└────────────────────────────────────────────────────────────┘
```

**Data flow:** The auditd watcher parses syscall events and runs them through behavior analysis, BarnacleDefense pattern matching, and policy evaluation *before* producing alerts. Other sources (sentinel, journald, scanner, firewall) produce alerts directly. All alerts flow through the `raw_tx` channel to the aggregator, which deduplicates and rate-limits before fanning out to Slack, TUI, REST API, and the hash-chained audit log. The admin socket accepts authenticated commands via Unix domain socket.

## Contributing

This is an early-stage project and contributions make a real difference right now — detection rules, scanners, integration guides, bug reports, or just telling me what's broken.

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for how to get started, including the CLA process and development guidelines.

## License

AGPL-3.0 — see [LICENSE](LICENSE) for details.

---

> 📚 **[Full Documentation Index →](docs/INDEX.md)**

---

<div align="center">

If you run AI agents on real infrastructure and want to help shape this, [get involved](CONTRIBUTING.md) or [file an issue](https://github.com/ClawTower/ClawTower/issues).

</div>
