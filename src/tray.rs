use anyhow::Result;
use crossbeam_channel::Sender;
use tray_item::{IconSource, TrayItem};

pub fn setup_tray(tx: Sender<()>) -> Result<TrayItem> {
    let mut tray = TrayItem::new("Hsarec", IconSource::Resource("#1"))?;
    tray.add_menu_item("开始拔线", move || {
        println!("开始拔线 - 菜单项点击");
        // TODO: Implement actual reconnect logic here if needed by the application
    })?;
    tray.inner_mut().add_separator()?;
    tray.add_menu_item("关于我", move || {
        let _ = webbrowser::open("https://blog.3gxk.net/about.html");
    })?;
    tray.add_menu_item("退出程序", move || {
        let _ = tx.send(());
    })?;
    Ok(tray)
}
