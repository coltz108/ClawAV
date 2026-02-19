# Hardcoded Key Auto-Remediation via Proxy

**Date:** 2026-02-19
**Status:** Approved design, pending implementation

---

## Problem

ClawTower's `scan:openclaw:hardcoded_secrets` scanner detects API keys hardcoded in OpenClaw config files (e.g., `xoxb-...` in `openclaw.json`). Currently it only alerts. The agent could read its own config to extract the real key, bypassing the proxy's virtual-key protection.

## Solution

When the scanner detects a hardcoded key, automatically:
1. Extract the full key value
2. Replace it in the config file with a proxy virtual key
3. Route the virtual key through ClawTower's existing API proxy
4. Notify via Slack/TUI
5. Track everything in a reversible manifest

This extends the proxy's network-layer key protection to the filesystem layer.

## Architecture

```
Scanner detects key
    |
    v
Remediation Engine (scanner/remediate.rs)
    |
    +-- 1. Extract full key value from config file
    +-- 2. Determine provider (JSON context -> prefix fallback)
    +-- 3. Generate virtual key (vk-remediated-<provider>-<short-hash>)
    +-- 4. Write remediation manifest (for reversibility)
    +-- 5. Rewrite config file (swap real key -> virtual key)
    +-- 6. Append KeyMapping to proxy config overlay (config.d/)
    +-- 7. Signal proxy to hot-reload mappings
    +-- 8. Emit CRIT alert with remediation details
```

### Decision: Auto-remediate + notify

No human approval gate. The operation is non-destructive (real key preserved in proxy mapping, encrypted backup in manifest). Notification goes through the normal alert pipeline to Slack/TUI.

### Decision: Scanner-adjacent, not response engine

The remediation lives in `scanner/remediate.rs`, called from the existing scanner when keys are found. The response engine's approval flow would add unnecessary latency for a safe, reversible operation.

---

## Components

### 1. Remediation Manifest

**File:** `/etc/clawtower/remediated-keys.json`

```json
{
  "version": 1,
  "remediations": [
    {
      "id": "a1b2c3d4",
      "timestamp": "2026-02-19T14:30:00Z",
      "source_file": "/home/openclaw/.openclaw/openclaw.json",
      "json_path": "channels.slack.botToken",
      "original_key_prefix": "xoxb-",
      "original_key_hash": "sha256:abcdef...",
      "encrypted_real_key": "base64:...",
      "encryption_salt": "hex:...",
      "virtual_key": "vk-remediated-slack-a1b2c3d4",
      "provider": "slack",
      "upstream": "https://slack.com/api"
    }
  ]
}
```

- **`original_key_hash`**: SHA-256 of the real key for integrity verification during restore.
- **`encrypted_real_key`**: AES-256-GCM encrypted backup of the real key. Encryption key derived from `/etc/machine-id` + per-entry salt. Machine-bound so the manifest alone is useless if exfiltrated.
- **`json_path`**: Exact location in the config file for targeted restore.

### 2. Provider Detection (Two-Stage)

**Stage 1 -- JSON context:** Parse the JSON structure and map field paths to providers.

| JSON Path Pattern | Provider | Upstream |
|---|---|---|
| `channels.slack.*Token` | `slack` | `https://slack.com/api` |
| `providers.anthropic.*` | `anthropic` | `https://api.anthropic.com` |
| `providers.openai.*` | `openai` | `https://api.openai.com` |
| `providers.groq.*` | `groq` | `https://api.groq.com` |
| `gateway.apiKey` | infer from prefix | -- |

**Stage 2 -- Prefix fallback:** If JSON context is ambiguous, map by key prefix.

| Prefix | Provider | Default Upstream |
|---|---|---|
| `sk-ant-` | `anthropic` | `https://api.anthropic.com` |
| `sk-proj-`, `sk-` | `openai` | `https://api.openai.com` |
| `gsk_` | `groq` | `https://api.groq.com/openai` |
| `xai-` | `xai` | `https://api.x.ai` |
| `xoxb-`, `xoxp-` | `slack` | `https://slack.com/api` |
| `ghp_` | `github` | `https://api.github.com` |
| `glpat-` | `gitlab` | `https://gitlab.com/api` |
| `AKIA` | `aws` | `https://sts.amazonaws.com` |

**Unknown providers:** Still remediated (key removed from config), but the notification flags that the proxy mapping needs manual upstream configuration.

### 3. Config File Rewriting

**openclaw.json:**
1. Read file, parse as `serde_json::Value`
2. Walk JSON tree to the detected path
3. Replace value with virtual key string
4. Write back with pretty-printing
5. Preserve file permissions and ownership (stat before, chown/chmod after)

**gateway.yaml:**
Targeted string replacement on the line containing the key to avoid reformatting the entire YAML structure.

### 4. Proxy Config Overlay

Write to `config.d/remediated-keys.toml` (not the base `config.toml` which may be immutable):

```toml
[[proxy.key_mapping]]
virtual_key = "vk-remediated-slack-a1b2c3d4"
real = "xoxb-the-actual-key-here"
provider = "slack"
upstream = "https://slack.com/api"
```

Leverages the existing config layering system (`config/merge.rs`) which appends list entries from overlays.

### 5. Proxy Hot-Reload

Add an `mpsc` reload channel to the proxy task. After writing the overlay, the remediation engine sends a reload signal. The proxy re-reads its config and rebuilds key mappings and credential states without restarting.

### 6. Notification

Alert emitted after successful remediation:

```
CRIT [scan:openclaw:remediated_secrets] Auto-remediated 1 hardcoded key in openclaw.json:
  xoxb-***->vk-remediated-slack-a1b2c3d4 (slack).
  Real key secured in proxy config. Run `clawtower restore-keys` to reverse.
```

Severity: CRIT (a hardcoded secret is a serious finding, even after remediation).

### 7. Restore Command

```
clawtower restore-keys              # Restore all remediated keys
clawtower restore-keys --id a1b2    # Restore a specific remediation
clawtower restore-keys --dry-run    # Show what would be restored
```

**Restore flow:**
1. Read manifest
2. For each entry:
   a. Read real key from proxy overlay
   b. Verify SHA-256 hash matches
   c. If proxy key missing, decrypt `encrypted_real_key` from manifest
   d. Read source config, find virtual key, replace with real key
   e. Write file, preserve permissions
   f. Remove KeyMapping from overlay
3. Remove manifest entries
4. Signal proxy to reload
5. Emit INFO alert: "Restored N keys to original locations"

**Uninstall integration:** Existing uninstall scripts call `clawtower restore-keys` as a pre-removal step.

---

## File Map

| Component | File | Change |
|---|---|---|
| Remediation engine | `src/scanner/remediate.rs` | **New** -- extraction, provider detection, rewriting, manifest |
| Scanner integration | `src/scanner/network.rs` | **Modify** -- call remediation from `scan_openclaw_hardcoded_secrets()` |
| Scanner module | `src/scanner/mod.rs` | **Modify** -- add `pub mod remediate;` |
| Proxy reload | `src/proxy/mod.rs` | **Modify** -- add reload channel, rebuild mappings on signal |
| CLI subcommand | `src/cli.rs` | **Modify** -- add `restore-keys` subcommand |
| CLI dispatch | `src/main.rs` | **Modify** -- handle `restore-keys` dispatch |
| Manifest | `/etc/clawtower/remediated-keys.json` | **Runtime** -- created on first remediation |
| Proxy overlay | `config.d/remediated-keys.toml` | **Runtime** -- created on first remediation |

---

## Edge Cases

- **Key already remediated:** Check manifest before remediating. If virtual key already exists for this path, skip (idempotent).
- **Config file not writable:** Alert with details, skip remediation for that key. Don't fail the entire scan.
- **Proxy not enabled:** Still rewrite the config (remove the hardcoded key) but warn that the virtual key won't resolve until proxy is enabled.
- **Multiple keys in one file:** Handle each independently with separate manifest entries and virtual keys.
- **File format changes between detection and rewrite:** Re-parse the file at rewrite time, verify the key is still present at the expected path.

---

## Testing

- Unit tests for provider detection (context + prefix, unknown fallback)
- Unit tests for JSON rewriting (round-trip preserves structure)
- Unit tests for manifest read/write/encryption/decryption
- Unit tests for restore logic (happy path + missing proxy key fallback)
- Integration test: plant a key in a temp config, run remediation, verify virtual key in file and real key in overlay
