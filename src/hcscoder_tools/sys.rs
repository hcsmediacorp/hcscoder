//! System introspection (host, OS, memory, CPUs) via `sysinfo`.

use anyhow::Result;
use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Serialize)]
pub struct SysSnapshot {
    pub hostname: String,
    pub os: String,
    pub kernel_version: String,
    pub long_os_version: String,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub cpus: usize,
    pub load_one: f64,
    pub load_five: f64,
    pub load_fifteen: f64,
}

/// Collect a stable snapshot of system state.
pub fn system_snapshot() -> Result<SysSnapshot> {
    let mut sys = System::new();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let load = System::load_average();

    Ok(SysSnapshot {
        hostname,
        os: System::name().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        long_os_version: System::long_os_version().unwrap_or_else(|| "unknown".to_string()),
        total_memory_bytes: sys.total_memory(),
        used_memory_bytes: sys.used_memory(),
        cpus: sys.cpus().len(),
        load_one: load.one,
        load_five: load.five,
        load_fifteen: load.fifteen,
    })
}

/// Dump environment variables (optional prefix filter on names).
pub fn env_dump(prefix_filter: Option<&str>) -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(k, _)| prefix_filter.map(|p| k.starts_with(p)).unwrap_or(true))
        .collect()
}
