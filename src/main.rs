#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut tray = TrayItem::new("Hsarec", IconSource::Resource("icon"))?;
    tray.add_menu_item("开始拔线", move || {
        println!("开始拔线");
    })?;
    tray.inner_mut().add_separator()?;
    tray.add_menu_item("关于我", move || {
        let _ = webbrowser::open("https://blog.3gxk.net/about.html");
    })?;
    tray.add_menu_item("退出程序", move || {
        let _ = tx.send(());
    })?;
    loop {
        if let Ok(_) = rx.try_recv() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}
