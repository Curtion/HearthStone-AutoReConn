use anyhow::Result;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub status: sysinfo::ProcessStatus,
}

pub fn get_process_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, false);

    let mut process_list: Vec<ProcessInfo> = Vec::new();

    for (pid, process) in sys.processes() {
        if process.name().to_string_lossy() == name {
            let proc_info = ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                status: process.status(),
            };
            process_list.push(proc_info);
        }
    }

    Ok(process_list)
}