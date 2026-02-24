// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 JR Morton

//! Embedded web dashboard server.
//!
//! Serves the Next.js static export from `dashboard/out/` via `rust-embed`,
//! alongside the existing API endpoints. Runs on a configurable port (default 1337).
//! Same-origin serving eliminates CORS between dashboard and API.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use rust_embed::Embed;

use super::api::{self, ApiContext};

#[derive(Embed)]
#[folder = "dashboard/out/"]
struct DashboardAssets;

/// Start the combined dashboard + API server.
pub async fn run_dashboard_server(
    bind: &str,
    port: u16,
    ctx: Arc<ApiContext>,
) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", bind, port).parse()?;

    let make_svc = make_service_fn(move |_conn| {
        let ctx = ctx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                dashboard_handle(req, ctx.clone())
            }))
        }
    });

    eprintln!("Dashboard server listening on {}", addr);
    Server::bind(&addr).serve(make_svc).await?;
    Ok(())
}

async fn dashboard_handle(
    req: Request<Body>,
    ctx: Arc<ApiContext>,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    // Delegate /api/* to the existing API handler (inject bearer token)
    if path.starts_with("/api/") {
        // Inject the bearer token so the dashboard doesn't need browser-side auth
        let (mut parts, body) = req.into_parts();
        if !ctx.auth_token.is_empty() {
            parts.headers.insert(
                hyper::header::AUTHORIZATION,
                hyper::header::HeaderValue::from_str(&format!("Bearer {}", ctx.auth_token))
                    .unwrap_or_else(|_| hyper::header::HeaderValue::from_static("")),
            );
        }
        let req = Request::from_parts(parts, body);
        return api::handle(req, ctx).await;
    }

    // Serve static files
    Ok(serve_static(path))
}

fn serve_static(request_path: &str) -> Response<Body> {
    // Strip leading slash and normalize
    let path = request_path.trim_start_matches('/');

    // Try exact file match
    if let Some(resp) = try_serve_file(path) {
        return resp;
    }

    // Try with index.html for directory paths (trailing slash)
    if path.is_empty() || path.ends_with('/') {
        let index_path = format!("{}index.html", path);
        if let Some(resp) = try_serve_file(&index_path) {
            return resp;
        }
    }

    // Try path/index.html for paths without trailing slash
    {
        let index_path = format!("{}/index.html", path);
        if let Some(resp) = try_serve_file(&index_path) {
            return resp;
        }
    }

    // SPA fallback: serve root index.html for unmatched non-file paths
    if !path.contains('.') {
        if let Some(resp) = try_serve_file("index.html") {
            return resp;
        }
    }

    // 404
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(Body::from("404 Not Found"))
        .unwrap()
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn try_serve_file(path: &str) -> Option<Response<Body>> {
    let file = DashboardAssets::get(path)?;
    let mime = content_type(path);
    let is_html = mime.starts_with("text/html");

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime);

    // Cache strategy: immutable for hashed _next/ assets, no-cache for HTML
    if path.starts_with("_next/") {
        builder = builder
            .header("Cache-Control", "public, max-age=31536000, immutable");
    } else if is_html {
        builder = builder
            .header("Cache-Control", "no-cache, no-store, must-revalidate");
    }

    Some(builder.body(Body::from(file.data.into_owned())).unwrap())
}
