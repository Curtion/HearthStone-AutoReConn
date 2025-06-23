use flume::{Sender, unbounded};
use log::{error, info};
use notify::Config;
use notify::PollWatcher;
use notify::{Event, EventKind, RecursiveMode, Result, Watcher};
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::net::Ipv4Addr;
use std::path::Path;
use std::path::PathBuf;

use crate::LOGFILE_NAME;
use crate::PROCESS_NAME;
use crate::network;
use crate::process;

#[derive(Debug)]
pub struct LogMessage {
    pub ip: Option<Ipv4Addr>,
    pub port: u16,
}

pub fn watch_log(log_tx: Sender<LogMessage>) -> anyhow::Result<()> {
    let (tx, rx) = unbounded::<Result<Event>>();

    let mut watcher = PollWatcher::new(
        move |res| {
            if let Err(e) = tx.send(res) {
                error!("监听日志文件通信异常: {:?}", e);
            }
        },
        Config::default().with_poll_interval(std::time::Duration::from_secs(1)),
    )?;

    let process_name = PROCESS_NAME;
    let data = process::get_process_by_name(process_name)?;
    if data.is_empty() {
        return Err(anyhow::anyhow!("没有找到名为 {} 的进程。", process_name));
    }
    if data.len() > 1 {
        return Err(anyhow::anyhow!(
            "找到多个名为 {} 的进程: {:?}",
            process_name,
            data
        ));
    }
    let path = data
        .get(0)
        .map(|p| p.path.clone())
        .ok_or_else(|| anyhow::anyhow!("无法获取进程 {} 的路径。", process_name))?;

    let path = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无法获取进程 {} 的日志文件路径。", process_name))?
        .join("Logs");

    let path = get_newest_folder(&path.to_string_lossy())?
        .ok_or_else(|| anyhow::anyhow!("无法找到最新的日志文件夹。"))?
        .join(LOGFILE_NAME);

    info!("正在监控日志文件: {:?}", path);

    // 记录文件的初始大小
    let mut last_size = fs::metadata(&path)?.len();

    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    for res in rx {
        let log_tx_clone = log_tx.clone();
        match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) => {
                    if let Err(e) = read_new_lines(log_tx_clone, &path, &mut last_size) {
                        error!("读取新增行时发生错误: {:?}", e);
                    }
                }
                _ => {}
            },
            Err(e) => {
                error!("日志文件监控发生错误: {:?}", e);
                continue;
            }
        }
    }

    Ok(())
}

pub fn reconnect(ip: Option<Ipv4Addr>, port: Option<u16>) -> anyhow::Result<()> {
    let process_name = PROCESS_NAME;
    let data = process::get_process_by_name(process_name)?;
    if data.is_empty() {
        return Err(anyhow::anyhow!("没有找到名为 {} 的进程。", process_name));
    }
    if data.len() > 1 {
        return Err(anyhow::anyhow!(
            "找到多个名为 {} 的进程: {:?}",
            process_name,
            data
        ));
    }
    let pid = data
        .get(0)
        .map(|p| p.pid)
        .ok_or_else(|| anyhow::anyhow!("无法获取进程 {} 的 PID。", process_name))?;
    let data = network::get_process_by_pid(pid)?;
    if data.is_empty() {
        return Err(anyhow::anyhow!(
            "没有找到与进程 {} (PID: {}) 相关的网络信息。",
            process_name,
            pid
        ));
    }
    let connections: Vec<String> = data.iter().map(|p| format!("{}", p)).collect();
    info!("获取到的网络信息: {}", connections.join(", "));

    data.iter()
        .find(|info| {
            if let (Some(ip), Some(port)) = (ip, port) {
                info.remote_addr_as_ipv4() == ip && info.remote_port_as_u16() == port
            } else {
                false
            }
        })
        .map_or_else(
            || Err(anyhow::anyhow!("没有找到匹配的网络信息。")),
            |info| {
                info!(
                    "正在关闭炉石网络连接 {}:{}",
                    info.remote_addr, info.remote_port
                );
                network::close_tcp_connection(info)?;
                Ok(())
            },
        )?;
    Ok(())
}

fn get_newest_folder(dir_path: &str) -> anyhow::Result<Option<PathBuf>> {
    let newest = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let path = entry.path();
            let created = entry.metadata().ok()?.created().ok()?;
            Some((path, created))
        })
        .max_by_key(|(_, created)| *created)
        .map(|(path, _)| path);

    Ok(newest)
}

fn read_new_lines(
    log_tx: Sender<LogMessage>,
    file_path: &Path,
    last_size: &mut u64,
) -> anyhow::Result<()> {
    let current_size = fs::metadata(file_path)?.len();

    if current_size > *last_size {
        let mut file = std::fs::File::open(file_path)?;
        file.seek(SeekFrom::Start(*last_size))?;

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let re = Regex::new(r"Network\.GotoGameServe\(\).*?address=\s([0-9.]+):(\d+)")?;
            if let Some(caps) = re.captures(&line) {
                if let (Some(ip_match), Some(port_match)) = (caps.get(1), caps.get(2)) {
                    if let Ok(port) = port_match.as_str().parse::<u16>() {
                        log_tx.send(LogMessage {
                            ip: ip_match.as_str().parse::<Ipv4Addr>().ok(),
                            port,
                        })?;
                    }
                }
            }
        }

        *last_size = current_size;
    }

    Ok(())
}
