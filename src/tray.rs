use anyhow::Result;
use crossbeam_channel::Sender;
use tray_item::{IconSource, TrayItem};

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Setting,
    Reconnect,
    Exit,
    UpdateMenu(String),
}

pub fn setup_tray(tx: Sender<TrayMessage>, reconnect_hotkey: &str) -> Result<TrayItem> {
    let reconnect_name = "开始拔线".to_string() + reconnect_hotkey;
    let mut tray = TrayItem::new("Hsarec", IconSource::Resource("#1"))?;
    let tx_clone = tx.clone();
    tray.add_menu_item(&reconnect_name, move || {
        let _ = tx_clone.send(TrayMessage::Reconnect);
    })?;
    tray.inner_mut().add_separator()?;
    let tx_setting = tx.clone();
    tray.add_menu_item("设置快捷键", move || {
        let _ = tx_setting.send(TrayMessage::Setting);
    })?;
    tray.add_menu_item("关于我", move || {
        let _ = webbrowser::open("https://blog.3gxk.net/about.html");
    })?;
    tray.add_menu_item("退出程序", move || {
        let _ = tx.send(TrayMessage::Exit);
    })?;
    Ok(tray)
}
