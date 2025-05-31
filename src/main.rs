#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyEvent, GlobalHotKeyManager
};
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

fn main() -> Result<()> {
    let manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Some(Modifiers::SHIFT), Code::KeyD);
    manager.register(hotkey)?;

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

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            println!("{:?}", event);
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}
