use ksni::{self, menu::*};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::process::{self, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const POLL_INTERVAL: Duration = Duration::from_secs(10);
const API_BASE: &str = "http://127.0.0.1:18791";

#[derive(Debug, Clone, Default)]
struct TrayState {
    running: bool,
    alerts_critical: u32,
    alerts_warning: u32,
    alerts_total: u32,
    last_scan_mins: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    #[serde(default)]
    alerts_critical: Option<u32>,
    #[serde(default)]
    alerts_warning: Option<u32>,
    #[serde(default)]
    alerts_total: Option<u32>,
    #[serde(default)]
    last_scan_epoch: Option<i64>,
}

/// Generate a 16x16 RGBA icon with a colored circle.
fn make_icon(r: u8, g: u8, b: u8) -> ksni::Icon {
    let size = 16i32;
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = 6.0f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;
            if dist <= radius {
                // ARGB32 format for ksni
                data[idx] = 255; // A
                data[idx + 1] = r;
                data[idx + 2] = g;
                data[idx + 3] = b;
            }
        }
    }
    ksni::Icon {
        width: size,
        height: size,
        data,
    }
}

fn green_icon() -> ksni::Icon {
    make_icon(76, 175, 80)
}

fn yellow_icon() -> ksni::Icon {
    make_icon(255, 193, 7)
}

fn red_icon() -> ksni::Icon {
    make_icon(244, 67, 54)
}

fn grey_icon() -> ksni::Icon {
    make_icon(158, 158, 158)
}

struct ClawAVTray {
    state: Arc<Mutex<TrayState>>,
}

impl ksni::Tray for ClawAVTray {
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let st = self.state.lock().unwrap();
        let icon = if !st.running {
            grey_icon()
        } else if st.alerts_critical > 0 {
            red_icon()
        } else if st.alerts_warning > 0 {
            yellow_icon()
        } else {
            green_icon()
        };
        vec![icon]
    }

    fn title(&self) -> String {
        "ClawAV".into()
    }

    fn id(&self) -> String {
        "clawav-tray".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let st = self.state.lock().unwrap();
        let status = if st.running { "Running" } else { "Stopped" };
        ksni::ToolTip {
            title: format!("ClawAV v{VERSION} — {status}"),
            description: format!("Alerts: {}", st.alerts_total),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let st = self.state.lock().unwrap();
        let status = if st.running { "Running" } else { "Stopped" };
        let last_scan = match st.last_scan_mins {
            Some(m) => format!("{m} min ago"),
            None => "N/A".into(),
        };
        let alerts = st.alerts_total;

        vec![
            StandardItem {
                label: format!("ClawAV v{VERSION} — {status}"),
                enabled: false,
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: format!("Alerts: {alerts}"),
                enabled: false,
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: format!("Last Scan: {last_scan}"),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Open TUI".into(),
                activate: Box::new(|_| {
                    let _ = Command::new("alacritty")
                        .args(["-e", "clawav"])
                        .spawn()
                        .or_else(|_| {
                            Command::new("x-terminal-emulator")
                                .args(["-e", "clawav"])
                                .spawn()
                        });
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Open Dashboard".into(),
                activate: Box::new(|_| {
                    let _ = Command::new("xdg-open")
                        .arg(API_BASE)
                        .spawn();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| {
                    process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn poll_status(client: &Client, state: &Arc<Mutex<TrayState>>) {
    let health_ok = client
        .get(format!("{API_BASE}/api/health"))
        .timeout(Duration::from_secs(5))
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !health_ok {
        let mut st = state.lock().unwrap();
        *st = TrayState::default();
        return;
    }

    let status: Option<StatusResponse> = client
        .get(format!("{API_BASE}/api/status"))
        .timeout(Duration::from_secs(5))
        .send()
        .ok()
        .and_then(|r| r.json().ok());

    let mut st = state.lock().unwrap();
    st.running = true;
    if let Some(s) = status {
        st.alerts_critical = s.alerts_critical.unwrap_or(0);
        st.alerts_warning = s.alerts_warning.unwrap_or(0);
        st.alerts_total = s.alerts_total.unwrap_or(0);
        st.last_scan_mins = s.last_scan_epoch.map(|epoch| {
            let now = chrono::Utc::now().timestamp();
            ((now - epoch).max(0) / 60) as u64
        });
    }
}

fn main() {
    let state = Arc::new(Mutex::new(TrayState::default()));
    let poll_state = Arc::clone(&state);

    // Background poller
    thread::spawn(move || {
        let client = Client::new();
        loop {
            poll_status(&client, &poll_state);
            thread::sleep(POLL_INTERVAL);
        }
    });

    let service = ksni::TrayService::new(ClawAVTray { state });
    service.run();
}
