use crossbeam_channel::unbounded;
use log::info;
use notify::{Event, RecursiveMode, Result, Watcher, recommended_watcher};
use std::path::Path;

use crate::process;
use crate::network;

pub fn watch_log() -> anyhow::Result<()> {
    let (tx, rx) = unbounded::<Result<Event>>();

    let mut watcher = recommended_watcher(move |res| {
        println!("回调执行线程ID: {:?}", std::thread::current().id());
        if let Err(e) = tx.send(res) {
            log::error!("监听日志文件通信异常: {:?}", e);
        }
    })?;

    watcher.watch(Path::new(r"D:\Code\Curtion"), RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => println!("event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

pub fn reconnect(process_name: &str) -> anyhow::Result<()> {
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
    let pid = data.get(0).map(|p| p.pid).ok_or_else(|| {
        anyhow::anyhow!("无法获取进程 {} 的 PID。", process_name)
    })?;
    let data = network::get_process_by_pid(pid)?;
    if data.is_empty() {
        return Err(anyhow::anyhow!(
            "没有找到与进程 {} (PID: {}) 相关的网络信息。",
            process_name,
            pid
        ));
    }
    info!("获取到的网络信息: {:#?}", data);
    Ok(())
}
