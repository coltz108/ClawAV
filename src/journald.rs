use anyhow::Result;
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::alerts::{Alert, Severity};
use crate::network::parse_iptables_line;

/// Check if journald is available on this system
pub fn journald_available() -> bool {
    std::process::Command::new("journalctl")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Tail kernel messages from journald for iptables log entries.
/// Spawns `journalctl -k -f -o json --since now` and parses JSON lines.
pub async fn tail_journald_network(
    prefix: &str,
    tx: mpsc::Sender<Alert>,
) -> Result<()> {
    let mut child = Command::new("journalctl")
        .args(["-k", "-f", "-o", "json", "--since", "now"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture journalctl stdout"))?;

    let mut reader = BufReader::new(stdout).lines();

    // Send startup notification
    let _ = tx.send(Alert::new(
        Severity::Info,
        "network",
        "Network monitor started (journald source)",
    )).await;

    while let Some(line) = reader.next_line().await? {
        // Parse JSON line from journalctl
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            // The kernel message is in the "MESSAGE" field
            if let Some(message) = json.get("MESSAGE").and_then(|v| v.as_str()) {
                if let Some(alert) = parse_iptables_line(message, prefix) {
                    let _ = tx.send(alert).await;
                }
            }
        }
    }

    // If journalctl exits, report it
    let _ = tx.send(Alert::new(
        Severity::Warning,
        "network",
        "journalctl process exited unexpectedly",
    )).await;

    Ok(())
}
