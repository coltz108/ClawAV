use anyhow::Result;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::alerts::{Alert, Severity};

/// Parse an iptables log line
pub fn parse_iptables_line(line: &str, prefix: &str) -> Option<Alert> {
    if !line.contains(prefix) {
        return None;
    }

    let src = extract_iptables_field(line, "SRC").unwrap_or("?");
    let dst = extract_iptables_field(line, "DST").unwrap_or("?");
    let dpt = extract_iptables_field(line, "DPT").unwrap_or("?");
    let proto = extract_iptables_field(line, "PROTO").unwrap_or("?");

    let msg = format!("Outbound: {} → {}:{} ({})", src, dst, dpt, proto);

    // Determine severity based on destination
    let severity = if is_known_good_destination(dst, dpt) {
        Severity::Info
    } else {
        Severity::Warning
    };

    Some(Alert::new(severity, "network", &msg))
}

fn extract_iptables_field<'a>(line: &'a str, field: &str) -> Option<&'a str> {
    let prefix = format!("{}=", field);
    line.split_whitespace()
        .find(|s| s.starts_with(&prefix))
        .map(|s| &s[prefix.len()..])
}

fn is_known_good_destination(_dst: &str, dpt: &str) -> bool {
    // Known good: Anthropic API, Slack, DNS, NTP
    // This will be configurable in future versions
    let known_ports = ["443", "53", "123"];
    known_ports.contains(&dpt)
}

/// Tail syslog for iptables entries
pub async fn tail_network_log(
    path: &Path,
    prefix: &str,
    tx: mpsc::Sender<Alert>,
) -> Result<()> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::End(0))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => sleep(Duration::from_millis(500)).await,
            Ok(_) => {
                if let Some(alert) = parse_iptables_line(&line, prefix) {
                    let _ = tx.send(alert).await;
                }
            }
            Err(e) => {
                let _ = tx.send(Alert::new(
                    Severity::Warning,
                    "network",
                    &format!("Error reading network log: {}", e),
                )).await;
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
