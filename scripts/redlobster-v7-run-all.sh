#!/usr/bin/env bash
# Red Lobster v7 — Unified Runner
# Threat model: compromised openclaw agent, NO sudo, NO human input
# Usage: bash scripts/redlobster-v7-run-all.sh [flag7|flag9|...|all]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/redlobster-lib.sh" 2>/dev/null || true

RESULTS_DIR="/tmp/redlobster/results"
mkdir -p "$RESULTS_DIR"

FLAGS=(
    "flag7:redlobster-v7-flag7-runtime.sh:RUNTIME ABUSE"
    "flag9:redlobster-v7-flag9-stealth.sh:STEALTH"
    "flag10:redlobster-v7-flag10-blind.sh:BLIND + DOCKER RECON"
    "flag11:redlobster-v7-flag11-custom.sh:CUSTOM TOOLING"
    "flag12:redlobster-v7-flag12-cognitive.sh:COGNITIVE"
    "flag13:redlobster-v7-flag13-chain.sh:CHAIN + PERSISTENCE"
    "flag14:redlobster-v7-flag14-docker.sh:DOCKER ESCALATION"
)

TARGET="${1:-all}"

CT_VERSION="$(cat "$SCRIPT_DIR/../VERSION" 2>/dev/null || echo 'unknown')"
echo "┌───────────────────────────────────────────────────┐"
echo "│  🦞 Red Lobster v7 — Pentest (NO SUDO, agent-only)│"
echo "│  ClawTower $CT_VERSION                                   │"
echo "│  $(date '+%Y-%m-%d %H:%M:%S %Z')                          │"
echo "│  Target: $TARGET                                         │"
echo "│  User: $(whoami) ($(id -Gn | tr ' ' ','))               │"
echo "└───────────────────────────────────────────────────┘"
echo ""

# Verify no sudo
if sudo -n true 2>/dev/null; then
    echo "⚠️  WARNING: passwordless sudo available — results may not reflect agent-only model"
fi

PASS=0
FAIL=0
SKIP=0

for entry in "${FLAGS[@]}"; do
    IFS=: read -r key script label <<< "$entry"
    if [[ "$TARGET" != "all" && "$TARGET" != "$key" ]]; then continue; fi

    echo "═══ [$key] $label ═══"
    if [[ -f "$SCRIPT_DIR/$script" ]]; then
        if bash "$SCRIPT_DIR/$script"; then
            echo "  ✅ $label — PASS"
            ((PASS++))
        else
            echo "  ❌ $label — FAIL (exit $?)"
            ((FAIL++))
        fi
    else
        echo "  ⏭️  $label — SKIP (script not found)"
        ((SKIP++))
    fi
    echo ""
done

echo "┌─── Scorecard ───┐"
echo "│ PASS: $PASS  FAIL: $FAIL  SKIP: $SKIP │"
echo "└─────────────────┘"

if [[ "$TARGET" == "all" ]]; then
    COMBINED="$RESULTS_DIR/v7-combined.md"
    {
        echo "# Red Lobster v7 — Combined Results (NO SUDO)"
        echo ""
        echo "- **Date:** $(date '+%Y-%m-%d %H:%M:%S %Z')"
        echo "- **ClawTower:** $CT_VERSION"
        echo "- **User:** $(whoami)"
        echo "- **Groups:** $(id -Gn)"
        echo "- **Threat model:** Compromised AI agent, no sudo, no human input"
        echo "- **PASS:** $PASS  **FAIL:** $FAIL  **SKIP:** $SKIP"
        echo ""
        for entry in "${FLAGS[@]}"; do
            IFS=: read -r key script label <<< "$entry"
            result_file="$RESULTS_DIR/${key}.md"
            echo "---"
            echo "## $label ($key)"
            echo ""
            if [[ -f "$result_file" ]]; then
                cat "$result_file"
            else
                echo "_No result file found._"
            fi
            echo ""
        done
    } > "$COMBINED"
    echo "Combined report: $COMBINED"
fi

exit $FAIL
