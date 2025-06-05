use anyhow::Result;
use sysinfo::{ProcessesToUpdate, System};

pub struct ProcessInfo {
    pub pid: sysinfo::Pid,
    pub name: String,
    pub status: sysinfo::ProcessStatus,
}

pub fn get_process_info() -> Result<Vec<ProcessInfo>> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, false);

    let mut process_list = Vec::new();

    for (pid, process) in sys.processes() {
        println!("{:#?}", process);
        let proc_info = ProcessInfo {
            pid: *pid,
            name: process.name().to_string_lossy().into_owned(),
            status: process.status(),
        };
        process_list.push(proc_info);
    }

    Ok(process_list)
}
