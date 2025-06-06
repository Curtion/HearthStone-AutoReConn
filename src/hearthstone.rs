use std::path::Path;

use crossbeam_channel::unbounded;
use notify::{Event, RecursiveMode, Result, Watcher, recommended_watcher};

pub fn watch_log() -> anyhow::Result<()> {
    let (tx, rx) = unbounded::<Result<Event>>();

    let mut watcher = recommended_watcher(move |res| {
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
