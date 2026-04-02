//! Network helpers: DNS resolution and TCP reachability.

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout as tokio_timeout;

/// Resolve host:port to socket addresses (async DNS).
pub async fn resolve_host(host: &str, port: u16) -> Result<Vec<SocketAddr>> {
    let addrs = tokio::net::lookup_host((host, port))
        .await
        .with_context(|| format!("DNS lookup failed for {}:{}", host, port))?;
    let v: Vec<SocketAddr> = addrs.collect();
    if v.is_empty() {
        anyhow::bail!("no addresses for {}:{}", host, port);
    }
    Ok(v)
}

/// TCP connect with timeout (milliseconds).
pub async fn tcp_probe(addr: SocketAddr, timeout_ms: u64) -> Result<()> {
    tokio_timeout(Duration::from_millis(timeout_ms), TcpStream::connect(addr))
        .await
        .context("connect timed out")?
        .context("TCP connect failed")?;
    Ok(())
}
