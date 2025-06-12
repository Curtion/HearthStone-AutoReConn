use std::path::PathBuf;

use anyhow::Result;
use sysinfo::{ProcessStatus, ProcessesToUpdate, System};

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub path: PathBuf,
}

pub fn get_process_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, false);

    let mut process_list: Vec<ProcessInfo> = Vec::new();

    for (pid, process) in sys.processes() {
        if process.name().to_string_lossy() == name && process.status() == ProcessStatus::Run {
            let proc_info = ProcessInfo {
                pid: pid.as_u32(),
                path: process.exe().map(|p| p.to_path_buf()).unwrap_or_default(),
            };
            process_list.push(proc_info);
        }
    }

    Ok(process_list)
}
