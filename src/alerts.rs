use chrono::{DateTime, Local};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}

impl Severity {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" | "crit" => Severity::Critical,
            "warning" | "warn" => Severity::Warning,
            _ => Severity::Info,
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            Severity::Info => "ℹ️",
            Severity::Warning => "⚠️",
            Severity::Critical => "🔴",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub timestamp: DateTime<Local>,
    pub severity: Severity,
    pub source: String,
    pub message: String,
}

impl Alert {
    pub fn new(severity: Severity, source: &str, message: &str) -> Self {
        Self {
            timestamp: Local::now(),
            severity,
            source: source.to_string(),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} [{}] {}",
            self.timestamp.format("%H:%M:%S"),
            self.severity,
            self.source,
            self.message
        )
    }
}

/// Ring buffer of alerts for the dashboard
pub struct AlertStore {
    alerts: Vec<Alert>,
    max_size: usize,
}

impl AlertStore {
    pub fn new(max_size: usize) -> Self {
        Self {
            alerts: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, alert: Alert) {
        if self.alerts.len() >= self.max_size {
            self.alerts.remove(0);
        }
        self.alerts.push(alert);
    }

    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }

    pub fn count_by_severity(&self, severity: &Severity) -> usize {
        self.alerts.iter().filter(|a| &a.severity == severity).count()
    }
}
