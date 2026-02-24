use std::path::Path;
use std::process::Command;

fn main() {
    // Rerun if dashboard sources change
    println!("cargo:rerun-if-changed=dashboard/src");
    println!("cargo:rerun-if-changed=dashboard/package.json");
    println!("cargo:rerun-if-changed=dashboard/next.config.ts");

    let out_dir = Path::new("dashboard/out");

    // Skip if already built (supports CI caching)
    if out_dir.join("index.html").exists() {
        return;
    }

    // Skip if env var set
    if std::env::var("CLAWTOWER_SKIP_DASHBOARD_BUILD").as_deref() == Ok("1") {
        create_fallback(out_dir);
        return;
    }

    // Check for node
    let has_node = Command::new("node").arg("--version").output().is_ok();
    if !has_node {
        eprintln!("cargo:warning=node not found — creating fallback dashboard");
        create_fallback(out_dir);
        return;
    }

    // Install deps if needed
    let node_modules = Path::new("dashboard/node_modules");
    if !node_modules.exists() {
        let status = Command::new("npm")
            .args(["install", "--prefix", "dashboard"])
            .status()
            .expect("failed to run npm install");
        if !status.success() {
            eprintln!("cargo:warning=npm install failed — creating fallback dashboard");
            create_fallback(out_dir);
            return;
        }
    }

    // Build
    let status = Command::new("npx")
        .args(["--prefix", "dashboard", "next", "build"])
        .current_dir("dashboard")
        .status()
        .expect("failed to run next build");
    if !status.success() {
        eprintln!("cargo:warning=next build failed — creating fallback dashboard");
        create_fallback(out_dir);
    }
}

fn create_fallback(out_dir: &Path) {
    std::fs::create_dir_all(out_dir).ok();
    std::fs::write(
        out_dir.join("index.html"),
        r#"<!DOCTYPE html><html><head><title>ClawTower</title></head>
<body><h1>ClawTower Dashboard</h1>
<p>Dashboard was not built. Run <code>cd dashboard && npm run build</code> and rebuild.</p>
</body></html>"#,
    ).ok();
}
