 #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
 
 use anyhow::Result;
 use std::sync::mpsc;
 use tray_item::{IconSource, TrayItem};
 use inputbot::KeybdKey;
 
 fn main() -> Result<()> {
     // 用于托盘图标退出事件
     let (tx, rx) = mpsc::channel();
 
     let mut tray = TrayItem::new("Hsarec", IconSource::Resource("icon"))?;
     tray.add_menu_item("开始拔线", move || {
         println!("开始拔线 - 菜单项点击");
     })?;
     tray.inner_mut().add_separator()?;
     tray.add_menu_item("关于我", move || {
         let _ = webbrowser::open("https://blog.3gxk.net/about.html");
     })?;
     let tx_clone_for_exit_menu = tx.clone();
     tray.add_menu_item("退出程序", move || {
         let _ = tx_clone_for_exit_menu.send(());
     })?;
 
     KeybdKey::F12Key.bind(|| {
         if KeybdKey::LControlKey.is_pressed() && KeybdKey::LShiftKey.is_pressed() {
             println!("全局热键 Ctrl+Shift+F12 被按下 (inputbot)");
         }
     });
     
     std::thread::spawn(|| {
         inputbot::handle_input_events();
     });

     match rx.recv() {
         Ok(_) => println!("收到退出信号，程序即将关闭。"),
         Err(_) => println!("退出信号通道发生错误，程序即将关闭。"),
     }
     
     Ok(())
 }
