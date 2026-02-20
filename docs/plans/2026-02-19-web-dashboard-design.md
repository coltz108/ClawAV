# ClawTower Web Dashboard Design

## Overview

A browser-based dashboard that mirrors the TUI experience, served alongside the existing ClawTower API. OpenClaw shares a pre-authenticated URL with the user, who logs in with their Linux credentials to access real-time alerts, scan results, and approval workflows.

## Architecture

```
Browser                    ClawTower (Rust, runs as root)
+----------------+         +-----------------------------+
|  Next.js 16    |--HTTPS->|  Existing API (:18791)      |
|  React 19      |         |  + POST /api/auth/login     |
|  Terminal UI   |<---WS---|  + GET  /api/ws              |
+----------------+         |  + POST /api/auth/validate   |
                           |  + Existing endpoints        |
                           +-----------------------------+
                                       |
                                  PAM auth
                                  (pam crate)
```

- Next.js app lives in `web/` at the repo root
- ClawTower Rust API is the sole backend (no BFF, no sidecar)
- WebSocket for real-time push (alerts, approvals, status)
- PAM authentication against Linux system users

## Auth Flow

### Two-factor by design

1. **API bearer token** (proves the user was given the URL by OpenClaw)
2. **Linux PAM login** (proves the user is an authorized human on the system)

### Sequence

1. OpenClaw generates URL: `https://host:18791/dashboard?token=<bearer_token>`
2. Next.js reads `?token`, stores in memory (not localStorage)
3. Redirects to `/login` — user enters Linux username + password
4. `POST /api/auth/login` validates via PAM, returns JWT (8h, httpOnly cookie)
5. **First-login check:** if `must_change_password: true` in response, redirect to `/change-password`
6. User sets new password via `POST /api/auth/change-password`, server clears the flag
7. Dashboard loads, opens WebSocket with JWT
8. Closing tab clears in-memory bearer token

### First-login password change

On fresh install, the installer generates a temporary Linux password for the admin user and sets a server-side `must_change_password` flag. The agent relays the temp password to the user (this is safe — it's one-time and must be changed immediately).

On first login:
- `/api/auth/login` succeeds but response includes `must_change_password: true`
- The UI blocks all navigation and shows a password change form
- `POST /api/auth/change-password` changes the Linux user's password and clears the flag
- Subsequent logins proceed directly to the dashboard

### Security properties

- Bearer token in URL only grants API access — still need PAM login
- JWT is short-lived (8h), httpOnly cookie (no JS access)
- JWT secret is random 256-bit, generated at startup, memory-only
- Restarts invalidate all sessions
- Failed logins rate-limited (5/IP/min) and logged as ClawTower alerts
- Only users in `clawtower-admin` group (or configured admin user) can log in
- Temporary password is one-time — must be changed on first login before accessing any dashboard functionality

## Rust API Additions

### POST /api/auth/login (auth-exempt)

```json
// Request
{ "username": "jr", "password": "..." }

// Response (success — normal login)
{ "token": "eyJhbG...", "expires_in": 28800, "username": "jr", "must_change_password": false }

// Response (success — first login, password change required)
{ "token": "eyJhbG...", "expires_in": 28800, "username": "jr", "must_change_password": true }

// Response (failure — after rate limit check)
{ "error": "invalid credentials" }
```

- Validates against Linux PAM (service name: `clawtower`)
- Only accepts users in `clawtower-admin` group or configured admin user
- Rate-limited: 5 attempts per IP per minute
- When `must_change_password` is true, the JWT is scoped to only allow `/api/auth/change-password`

### POST /api/auth/change-password (requires JWT)

```json
// Request
{ "old_password": "...", "new_password": "..." }

// Response (success)
{ "token": "eyJhbG...", "expires_in": 28800, "username": "jr", "must_change_password": false }

// Response (failure)
{ "error": "old password incorrect" }
```

- Verifies old password via PAM, then changes the Linux user's password
- Clears the server-side `must_change_password` flag
- Issues a new full-access JWT (replaces the scoped one)

### POST /api/auth/validate (requires JWT)

```json
// Response
{ "valid": true, "username": "jr", "expires_at": "2026-02-20T01:30:00Z", "must_change_password": false }
```

### GET /api/ws (requires JWT in query param)

```
ws://host:18791/api/ws?jwt=eyJhbG...
```

Taps into the same channels the TUI uses internally.

**Server -> Client:**

```json
{"type": "alert",             "data": {"ts": "...", "severity": "CRIT", "source": "auditd", "message": "..."}}
{"type": "approval",          "data": {"id": "uuid", "command": "...", "agent": "clawsudo", "severity": "warning", "context": "...", "created_at": "...", "timeout_secs": 300, "status": "pending"}}
{"type": "approval_resolved", "data": {"id": "uuid", "status": "approved", "by": "jr", "via": "slack"}}
{"type": "status",            "data": {"uptime_seconds": 3600, "risk_level": "nominal", "modules": {...}}}
{"type": "scan",              "data": {"category": "process_health", "status": "Pass", "details": "..."}}
{"type": "ping"}
```

**Client -> Server:**

```json
{"type": "resolve_approval", "data": {"id": "uuid", "approved": true, "message": "optional note"}}
{"type": "pong"}
```

Reconnection: exponential backoff (1s, 2s, 4s, 8s, max 30s).

### New crate dependencies

| Crate | Purpose |
|-------|---------|
| `pam` | Linux PAM authentication |
| `jsonwebtoken` | JWT sign/verify (HS256) |
| `tokio-tungstenite` | WebSocket upgrade in hyper handler |

## Frontend Structure

```
web/
  app/
    layout.tsx              # Root — terminal font, dark theme
    login/page.tsx          # Linux username/password login
    change-password/page.tsx # First-login forced password change
    dashboard/
      layout.tsx            # Auth shell — tab bar, status bar, WS
      page.tsx              # Overview (TUI Tab 0)
      alerts/page.tsx       # Alerts (TUI Tab 1)
      commands/page.tsx     # Commands (TUI Tab 2)
      network/page.tsx      # Network (TUI Tab 3)
      fim/page.tsx          # File Integrity (TUI Tab 4)
      scans/page.tsx        # Scans (TUI Tab 5)
      approvals/page.tsx    # NEW — approval cards with countdown
    globals.css             # Terminal theme, scan-lines
  components/
    AlertTable.tsx          # Scrollable alert list, severity colors
    ApprovalCard.tsx        # Approve/Deny with live countdown
    StatusBar.tsx           # Bottom bar: uptime, version, risk, WS
    TabNav.tsx              # Tab navigation (TUI-style)
    ScanResults.tsx         # Pass/Warn/Fail grouped display
    ModuleGrid.tsx          # Monitor status grid
    Terminal.tsx            # Shared terminal container
  lib/
    api.ts                  # REST client with JWT
    ws.ts                   # WebSocket manager
    auth.ts                 # JWT storage, login, redirect
  package.json
  next.config.ts
  tailwind.config.ts
```

### Page-to-TUI mapping

| Web Page | TUI Tab | Data Source |
|----------|---------|-------------|
| `/dashboard` | Tab 0 (Dashboard) | `/api/status` + `/api/security` + `/api/scans` + WS |
| `/dashboard/alerts` | Tab 1 (Alerts) | `/api/alerts` + WS stream |
| `/dashboard/commands` | Tab 2 (Commands) | WS stream (filter: auditd, behavior, policy, barnacle) |
| `/dashboard/network` | Tab 3 (Network) | WS stream (filter: network, ssh, firewall) |
| `/dashboard/fim` | Tab 4 (FIM) | WS stream (filter: sentinel, samhain, cognitive) |
| `/dashboard/scans` | Tab 5 (Scans) | `/api/scans` + periodic refresh |
| `/dashboard/approvals` | NEW | `/api/pending` + WS approval events |

No Config editor tab in v1 (TUI config editor requires sudo — too complex for web).

### Dependencies

```json
{
  "dependencies": {
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "tailwindcss": "^4.0.0"
  }
}
```

No state management library — React 19 `use()` + `useReducer` for WebSocket events.
No component library — hand-built terminal aesthetic.

## Visual Design

### Color palette

```
Background:   #0a0a0a  (near-black)
Surface:      #1a1a1a  (card/panel backgrounds)
Border:       #333333  (box-drawing: ╔═╗║╚═╝)
Text:         #c0c0c0  (default, slightly dim)
Text bright:  #e0e0e0  (headings, selected)
Green:        #00ff41  (Pass, healthy, connected)
Amber:        #ffb000  (Warning, degraded)
Red:          #ff2b2b  (Critical, error, denied)
Blue:         #00aaff  (Info, links, active tab)
Dim:          #666666  (timestamps, secondary)
```

### Typography

`JetBrains Mono` primary, `Fira Code` fallback, `monospace`. No proportional fonts.

### Visual elements

- Box-drawing borders around every panel (`╔══╗ ║ ╚══╝`)
- ASCII status indicators: `[●]` active, `[○]` inactive, `[!]` warning
- Severity badges as colored text: `CRIT`, `WARN`, `INFO`
- Scan-line overlay: CSS `repeating-linear-gradient` at 2px intervals, 3% opacity
- Cursor blink on login input fields
- Tab bar: `[ Dashboard | Alerts | Commands | Network | FIM | Scans | Approvals ]` with `>` active
- Status bar (fixed bottom): `CLAWTOWER v0.5.6 | ^ 3h 22m | Risk: NOMINAL | WS: * CONNECTED`
- Alert rows: `> ` prefix on hover/selected, same as TUI
- Lobster ASCII art on overview page

### Approval card with live countdown

```
+----------------------------------------------+
|  ! APPROVAL REQUEST                    3:42  |
|----------------------------------------------|
|  Command:  systemctl restart nginx           |
|  Agent:    clawsudo                          |
|  Severity: WARNING                           |
|  Context:  Apply nginx configuration changes |
|                                              |
|       [ APPROVE ]        [ DENY ]            |
+----------------------------------------------+
```

Countdown behavior:
- **>= 60s**: green timer
- **< 60s**: amber, border pulses faster
- **< 15s**: red, urgent pulse
- **0:00**: card dims, buttons disabled, shows `TIMED OUT`

Calculated client-side from `created_at + timeout_secs`.

## OpenClaw Skill Update

Add to `skill/SKILL.md` under a new "Web dashboard" section:

```
## Web dashboard

After installation, share the dashboard URL and temporary password with the user:

> Your ClawTower dashboard is ready:
> https://<host>:18791/dashboard?token=<api_bearer_token>
>
> Log in with username `<admin_user>` and temporary password `<temp_password>`.
> You'll be asked to set a new password on first login.

The API bearer token is in /etc/clawtower/config.toml under [api].auth_token.
You have read access to this value via clawsudo cat.

The temporary password is returned in the installer output. It is safe to relay
to the user — it's one-time and must be changed immediately on first login.
```

## Installer Changes

- Generate a random temporary password (12 alphanumeric characters)
- Set as the admin Linux user's password via `chpasswd`
- Write `must_change_password = true` to a server-side state file
- Return the temp password in the installer output (agent relays to user)
- Create `clawtower-admin` group if it doesn't exist, add admin user to it

## Open Questions (for implementation time)

- **TLS:** Self-signed cert, Let's Encrypt, or localhost-only? If accessed over the network, TLS is required.
- **Bearer token in URL:** Visible in browser history. Consider a short-lived exchange — POST the token to get a session cookie, then strip from URL via redirect.
- **State file location:** Where to persist `must_change_password` flag (e.g., `/var/lib/clawtower/state.json`)
- **Password complexity:** Minimum requirements to enforce during change
- **Session expiry:** Whether to support "remember me" or always 8h
- **Static file serving:** Does ClawTower's hyper server serve the Next.js build output, or is Next.js served separately? If same-origin, CORS is not needed.

## Build Order

1. Rust: PAM login endpoint + JWT signing (with `must_change_password` flag)
2. Rust: Change-password endpoint
3. Rust: JWT validate endpoint
4. Rust: WebSocket endpoint (tap into alert/approval channels)
5. Installer: temp password generation + `must_change_password` flag
6. Next.js: scaffold, terminal theme, login page
7. Next.js: change-password page (first-login flow)
8. Next.js: dashboard layout + WebSocket manager
9. Next.js: Overview page (status, module grid, recent alerts, lobster art)
10. Next.js: Alerts page (real-time stream + search/filter)
11. Next.js: Approvals page (live countdown cards, approve/deny over WS)
12. Next.js: Commands, Network, FIM, Scans pages
13. Skill update: teach OpenClaw to share the dashboard URL + temp password
14. Integration test: full flow — install -> relay temp password -> first login -> change password -> approve clawsudo command
