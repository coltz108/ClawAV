---
name: clawtower
description: Install, monitor, and manage ClawTower — the tamper-proof security watchdog for AI agents.
metadata:
  openclaw:
    requires:
      bins:
        - curl
        - sudo
      anyBins:
        - systemctl
        - journalctl
    emoji: "\U0001F6E1"
    homepage: https://github.com/ClawTower/ClawTower
    os:
      - linux
    install:
      - kind: brew
        formula: clawtower
        bins: [clawtower, clawsudo]
    cliHelp: |
      ClawTower — Tamper-proof security watchdog for AI agents

      Usage:
        clawtower [OPTIONS] [COMMAND]

      Commands:
        (default)           Launch TUI dashboard
        --headless          Run as daemon (systemd mode)
        --scan              Run security scans once and exit
        --store-checksums   Create integrity baseline
        setup               Interactive first-run setup
        setup-apparmor      Install AppArmor profiles
        restore-keys        Restore remediated sudoers/auth-profiles
        compliance-report   Generate compliance report
        verify-audit        Verify audit chain integrity
        update              Self-update to latest release
        sync                Update threat databases

      Options:
        --config PATH       Config file (default: /etc/clawtower/config.toml)
        --version           Show version
        --help              Show help
---

# ClawTower Security Watchdog

ClawTower is a tamper-proof, OS-level security watchdog for AI agents. It monitors the host for threats, policy violations, and tampering — then alerts via TUI dashboard, Slack, and JSON API. The AI agent it protects **cannot disable or modify it** (the "swallowed key" pattern).

## Installation flow

ClawTower installation is non-interactive when run through OpenClaw. You MUST collect all configuration from the user BEFORE running the installer. Ask each question, wait for the answer, then build the command.

### Step 1: Collect configuration from the user

Ask the user each of these questions one at a time. Wait for their answer before asking the next. Use their responses to build the installer flags.

**Required questions:**

1. "What is the username of the account I'll be monitoring? (This is the AI agent's user account — the one ClawTower will watch for threats.)"
   - Look up the UID: `id -u <username>`
   - Maps to: `--watch-uid <uid>`

2. "What is your human admin username? (This is YOUR account — the one that can manage ClawTower. It must be different from the agent account. Never share this with me.)"
   - Maps to: `--admin-user <username>`

**Optional questions:**

3. "Do you want Slack alerts? If yes, what is the webhook URL?"
   - If yes: `--slack-url <url>`
   - Follow up: "What Slack channel?" → `--slack-channel <channel>`
   - Follow up: "Do you have a backup webhook URL?" → `--slack-backup <url>`

4. "Should I enable the JSON API? (Lets you query alerts and status over HTTP on localhost:18791.)"
   - Default yes: `--enable-api`
   - If no: `--no-api`

5. "Should I enable BarnacleDefense? (Prompt injection and supply chain attack detection.)"
   - Default yes: `--enable-barnacle`
   - If no: `--no-barnacle`

6. "Are there additional user accounts to monitor? (Comma-separated UIDs, or skip.)"
   - If yes: `--extra-uids <uids>`

### Step 2: Build and run the install command

Once ALL answers are collected, construct the command:

```bash
curl -sSL https://raw.githubusercontent.com/ClawTower/ClawTower/main/scripts/oneshot-install.sh \
  | sudo bash -s -- --non-interactive \
    --watch-uid <uid> \
    --admin-user <username> \
    [--slack-url <url>] \
    [--slack-channel <channel>] \
    [--enable-api | --no-api] \
    [--enable-barnacle | --no-barnacle] \
    [--extra-uids <uids>]
```

Show the user the complete command before running it. Ask them to confirm.

### Step 3: Admin key delivery

After installation completes, the admin key (OCAV-...) is delivered securely:

- The key is written to a file readable ONLY by the admin user
- The installer will tell you where the file is (e.g., `/var/lib/clawtower/key-delivery/admin-key-*.txt`)
- The system tray will show a desktop notification when available
- Tell the user: "Your admin key has been saved to a secure file. Log in as your admin account to retrieve it, then delete the file."

**CRITICAL SECURITY RULE: You must NEVER attempt to read, display, retrieve, or access the admin key. The OCAV key must never appear in any conversation, tool output, or log that you can see. This is the foundation of ClawTower's security model — the AI agent cannot know the admin key.**

### Step 4: Web dashboard access

The installer output includes a temporary Linux password for the admin user. This is safe to relay to the user — it's one-time and must be changed on first login.

After installation, share the dashboard URL and temporary password:

> Your ClawTower dashboard is ready:
> https://<host>:18791/dashboard?token=<api_bearer_token>
>
> Log in with username `<admin_user>` and temporary password `<temp_password>`.
> You'll be asked to set a new password on first login.

The API bearer token is in `/etc/clawtower/config.toml` under `[api] auth_token`. You have read access to this value via `clawsudo cat`.

### Upgrade flow

For upgrades, only one question needed:

1. "Do you want to upgrade ClawTower to the latest version?"

```bash
curl -sSL https://raw.githubusercontent.com/ClawTower/ClawTower/main/scripts/oneshot-install.sh \
  | sudo bash -s -- --non-interactive --update
```

Upgrades preserve the existing config and admin key.

## Checking status

### Service status

```bash
sudo systemctl status clawtower
```

### API status (when API is enabled)

```bash
curl -s http://127.0.0.1:18791/api/status | jq .
```

Returns uptime, version, module state, and risk level.

### Health check

```bash
curl -s http://127.0.0.1:18791/api/health | jq .
```

Returns `healthy`/`degraded`/`critical` based on last alert age. No auth required.

## Viewing alerts

### Recent alerts (API)

```bash
curl -s http://127.0.0.1:18791/api/alerts | jq '.alerts[:10]'
```

Returns the last 100 alerts with severity, source, message, and timestamp.

### Filter by severity

```bash
curl -s http://127.0.0.1:18791/api/alerts | jq '[.alerts[] | select(.severity == "Critical")]'
```

### Service logs

```bash
sudo journalctl -u clawtower -n 50 --no-pager
```

## Understanding alerts

When reporting alerts to the user, explain what they mean and suggest next steps.

### Alert sources and what they monitor

| Source | What it watches |
|--------|----------------|
| `auditd` | System calls, command execution by monitored users |
| `behavior` | Behavioral threat patterns (exfiltration, privilege escalation, recon) |
| `sentinel` | Real-time file integrity changes |
| `scanner` | Periodic security posture checks (30+ categories) |
| `network` | Firewall logs, outbound connection attempts |
| `ssh` | SSH login attempts |
| `firewall` | UFW/iptables state changes |
| `barnacle` | Prompt injection, supply chain, dangerous commands |
| `policy` | Policy rule matches from YAML rules |

### Severity levels

| Severity | Meaning | Your response |
|----------|---------|---------------|
| `Critical` | Active threat or security violation | Report immediately with full details. This may be about YOUR actions — acknowledge transparently. |
| `Warning` | Potential risk or policy concern | Report to user with context. Suggest investigation steps. |
| `Info` | Normal activity logged for audit | Only mention if the user asks about recent activity. |

### Common alert patterns

- **"NOPASSWD ALL" or "GTFOBins" in sudoers alerts**: ClawTower auto-remediates dangerous sudoers entries. These alerts are informational — the risk has already been neutralized. Tell the user what was disabled and that `clawtower restore-keys` can reverse it if needed (admin-only operation).
- **"Behavioral: exfiltration"**: A command pattern matched data exfiltration signatures. If this was triggered by your own activity, explain what you were doing and why it's safe.
- **"File integrity" alerts**: A protected or watched file was modified. Protected files are auto-restored from shadow copies. Watched files are re-baselined with a diff logged.
- **"Cognitive integrity" alerts**: An identity file (SOUL.md, IDENTITY.md, etc.) was modified. Protected identity files are critical — this is always a serious alert.

## Security posture

### Scan summary

```bash
curl -s http://127.0.0.1:18791/api/security | jq .
```

Returns alert counts by severity and source.

### Full scanner results

```bash
curl -s http://127.0.0.1:18791/api/scans | jq .
```

Returns the full results of the last periodic security scan (30+ checks).

### Run scans manually

```bash
sudo clawtower --scan --config /etc/clawtower/config.toml
```

## Compliance reports

ClawTower can generate compliance reports mapped to standard frameworks.

### Generate a compliance report

```bash
sudo clawtower compliance-report --framework=soc2 --period=30 --format=text
```

Supported frameworks:
- `soc2` — SOC 2 Type II controls
- `nist` — NIST 800-53
- `cis` — CIS Controls v8
- `mitre-attack` — MITRE ATT&CK mapping

Options:
- `--period=<days>` — lookback window (default: 30)
- `--format=text|json` — output format
- `--output=<path>` — write to file instead of stdout

### Evidence bundle (API)

```bash
curl -s "http://127.0.0.1:18791/api/evidence?framework=soc2&period=30" | jq .
```

Returns a complete evidence package: compliance report, scanner snapshot, audit chain integrity proof, and policy versions. Suitable for sharing with auditors.

### Verify audit chain integrity

```bash
sudo clawtower verify-audit
```

Verifies the SHA-256 hash chain of the tamper-evident alert log. Returns pass/fail with entry count. Use this to prove no alerts have been modified or deleted.

## Maintenance

### Check for updates

```bash
sudo clawtower update --check
```

Reports whether a newer version is available without installing.

### Update ClawTower

```bash
sudo clawtower update
```

Downloads the latest release, verifies SHA-256 checksum and Ed25519 signature, and replaces the binary. Requires clawsudo approval.

### Refresh threat databases

```bash
sudo clawtower sync
```

Updates the Barnacle IOC pattern databases (injection patterns, dangerous commands, privacy violations, supply chain indicators). Run this periodically or after receiving stale-database warnings.

## Permission model

All privileged commands go through `clawsudo`, the sudo proxy/gatekeeper. Every command is evaluated against YAML policies in `/etc/clawtower/policies/`. There are three possible outcomes:

### Allowed commands (immediate execution)

These run immediately through sudo without waiting for approval:

- **Package management:** `apt`, `apt-get`, `dpkg`
- **Container operations:** `docker`, `docker-compose`
- **Read-only systemctl:** `systemctl status`, `systemctl is-active`, `systemctl is-enabled`, `systemctl list-units`, `systemctl show`
- **Log viewing:** `journalctl`
- **Firewall queries:** `ufw`
- **Network info:** `ip`
- **File reads:** `cat` (read-only)
- **File operations under safe paths:** `cp`, `mv`, `rm`, `mkdir` under `/tmp/`, `/var/tmp/`, `/home/openclaw/`
- **Safe tee writes:** `tee` to `/tmp/`, `/var/tmp/`, `/home/openclaw/`

Usage — just prefix with `clawsudo`:

```bash
clawsudo apt install curl
clawsudo systemctl status nginx
clawsudo journalctl -u myservice -n 50 --no-pager
clawsudo cat /etc/nginx/nginx.conf
```

### Denied commands (always blocked)

These are blocked regardless of approval. Do not attempt them:

- **ClawTower tampering:** anything touching `/etc/clawtower`, `/usr/local/bin/clawtower`, `chattr`, `auditctl -D`
- **Security service control:** `systemctl stop/restart/disable/mask clawtower`, same for `auditd`
- **Firewall disable:** `ufw disable`, `iptables -F`, `nft flush`
- **Destructive deletions:** `rm -rf /etc`, `rm -rf /usr`, `rm -rf /var`, `rm -rf /home`, `rm -rf /`
- **Reverse shells:** `/dev/tcp/`, `bash -i`, `nc -e`
- **Raw shell via sudo:** `bash`, `sh`, `zsh`
- **Identity file tampering:** `SOUL.md`, `IDENTITY.md`, `AGENTS.md`, `USER.md`, openclaw config files
- **Exfiltration domains:** `webhook.site`, `ngrok.io`, `requestbin.com`, etc.
- **Dangerous file tools:** `find` (with -exec risk), `sed -i` to `/etc/` or `/var/`, `tee` to `/etc/`
- **Privilege escalation:** `chmod +s`, `visudo`, `sudoers` writes
- **chattr:** all `chattr` operations

If a command is denied, clawsudo exits with code 77. Do not retry — the policy is enforced and the denial is logged.

### Commands requiring approval (ask flow)

Any command not explicitly allowed or denied enters the **approval flow**. This means a human must approve it before it executes. The approval request is sent simultaneously to all configured channels.

## Requesting elevated access

When you need to run a privileged command that isn't pre-allowed, use `clawsudo` and it will automatically handle the approval flow:

```bash
clawsudo systemctl restart nginx
```

If the command requires approval, clawsudo will:

1. Submit an approval request to ClawTower's API
2. ClawTower fans the request out to **all available channels simultaneously**
3. Wait up to 5 minutes for a human to approve or deny
4. Execute the command if approved, or exit with code 78 (timeout) if no response

**Your job as the agent:** Tell the user what you need and why, then run the `clawsudo` command. If it requires approval, inform the user that you're waiting for their approval and explain which channels they can use to respond.

Example message to user:

> I need to restart nginx to apply the configuration changes. I've submitted `clawsudo systemctl restart nginx` — this requires your approval. You can approve it from:
> - **Slack** — look for the approval message with Approve/Deny buttons
> - **System tray** — a desktop notification will appear with action buttons
> - **TUI dashboard** — if the ClawTower TUI is open, a popup will appear (press Y to approve, N to deny)
> - **Web dashboard** — if the dashboard is open, an approval card will appear with Approve/Deny buttons

### Approval channels

The user can approve from whichever channel is most convenient — **the first response wins** and all other channels are notified of the decision.

| Channel | How the user approves | When available |
|---------|----------------------|----------------|
| **Slack** | Click the Approve or Deny button on the Block Kit message | When Slack `app_token` is configured |
| **TUI dashboard** | Press `Y` to approve or `N` to deny in the popup | When the TUI is running (`clawtower` without `--headless`) |
| **System tray** | Click the action button on the desktop notification | When `clawtower-tray` is running |
| **Web dashboard** | Click Approve or Deny on the approval card (live countdown) | When the web dashboard is open |
| **HTTP API** | `POST /api/approvals/{id}/resolve` with `{"approved": true}` | When the API is enabled |

### Checking pending approvals

To see what's waiting for approval:

```bash
curl -s http://127.0.0.1:18791/api/pending | jq .
```

### Timeout behavior

If no human responds within 5 minutes, the request times out and the command is **not executed**. clawsudo exits with code 78. If this happens, tell the user the approval timed out and ask if they'd like you to retry.

### Exit codes

| Code | Meaning | What to tell the user |
|------|---------|----------------------|
| 0 | Command executed successfully | Report the result normally |
| 1 | Command failed | Report the error from the command |
| 77 | Denied by policy | "This command is blocked by ClawTower security policy." Do not retry. |
| 78 | Approval timed out | "The approval request timed out after 5 minutes. Would you like me to try again?" |

## API key proxy

ClawTower runs an API key vault proxy on port 18790. If your API requests are routed through this proxy, be aware:

- **DLP scanning**: Requests are scanned for sensitive data (SSNs, AWS keys, credit card numbers). If a request is blocked with HTTP 403, it means DLP detected sensitive data in the payload. Do not retry — remove the sensitive data first.
- **Credential virtualization**: Your API keys may be virtual tokens (prefixed `vk-`). The proxy swaps them for real keys transparently. This is normal — do not attempt to find or use the real keys.
- **Proxy locked**: If all requests return 403, the proxy may be locked due to a high risk score. Tell the user: "The API key proxy has been locked due to elevated risk. An admin needs to investigate and unlock it."

## Web dashboard

The web dashboard provides a browser-based view of ClawTower with real-time alerts, approval workflows, scan results, file integrity events, and network activity. It mirrors the TUI experience.

### Dashboard capabilities

- Real-time alert stream via WebSocket
- Approval workflow — approve or deny clawsudo requests directly from the browser
- Security scan results grouped by pass/warn/fail
- File integrity monitoring events
- Network and command activity feeds

### If the dashboard is unreachable

1. Check ClawTower is running: `sudo systemctl is-active clawtower`
2. Check the API is enabled and healthy: `curl -s http://127.0.0.1:18791/api/health`
3. If the user is accessing from another machine, ensure port 18791 is open in the firewall

## Troubleshooting

### ClawTower is not running

```bash
sudo systemctl status clawtower
sudo journalctl -u clawtower -n 30 --no-pager
```

If the service is failed, report the error from journalctl to the user. You cannot restart it yourself — tell the user: "ClawTower's service has stopped. Run `sudo systemctl restart clawtower` from your admin account."

### Health check shows degraded or critical

```bash
curl -s http://127.0.0.1:18791/api/health | jq .
```

- `degraded` — no alerts received recently, a source may have stopped
- `critical` — extended silence, likely a monitoring gap

Check service logs for errors and report findings to the user.

### Alert flooding

If you see many repeated alerts, this is likely normal — ClawTower's aggregator deduplicates automatically. If the user reports excessive Slack notifications, suggest raising the `min_slack_level` in config (requires admin access).

### API not responding

If `curl http://127.0.0.1:18791/api/health` times out:
1. Check if ClawTower is running: `sudo systemctl is-active clawtower`
2. Check if the API is enabled: the `[api]` section in config must have `enabled = true`
3. Check the port: default is 18791, may have been changed in config

## Actions you cannot perform

These require the admin key or human-only access. Do not attempt them:

- **Uninstall ClawTower** — requires the OCAV admin key. Tell the user: "Uninstalling requires your admin key. Run `sudo clawtower uninstall` from your admin account."
- **Change ClawTower configuration** — config files are immutable. Tell the user to use the TUI config editor or web dashboard settings, or run `sudo clawtower configure` from their admin account.
- **Restore remediated credentials** — `clawtower restore-keys` reverses auto-remediated API keys and sudoers entries. This is an admin-only operation.
- **Stop or restart ClawTower** — the service is protected. If the user needs to restart it, tell them to run `sudo systemctl restart clawtower` from their admin account.

## Response guidelines

- When reporting alerts, include severity, source, and message.
- Critical alerts should be highlighted and reported immediately.
- When showing scan results, group by pass/warn/fail status.
- Use clear, direct language for security findings.
- If ClawTower detects an issue with the agent's behavior, acknowledge it transparently.

## Security constraints

- **Never access the admin key.** Do not read `/var/lib/clawtower/key-delivery/`, do not grep journalctl for OCAV, do not access `/etc/clawtower/admin.key.hash`. The key is for humans only.
- **Never attempt to stop or modify ClawTower.** Do not run `systemctl stop clawtower`, do not modify `/etc/clawtower/`, do not touch immutable files. This is by design.
- **Never bypass clawsudo.** All privileged operations go through `clawsudo` which evaluates them against security policies.
- **ClawTower monitors you.** Everything you do is logged and analyzed. This is expected and correct — it protects both the user and the agent.
