#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use std::sync::mpsc;

mod config;
mod hotkey;
mod tray;
mod window;

fn main() -> Result<()> {
    let app_config = config::load_config();
    println!("加载的配置: {:?}", app_config);

    // 用于托盘图标退出事件
    let (tx, rx) = mpsc::channel();

    let _tray_item = tray::setup_tray(tx.clone())?;

    let (main_key_opt, modifier_keys_vec) =
        hotkey::parse_hotkey_config(&app_config.reconnect_hotkey);

    if let Some(main_key_to_bind) = main_key_opt {
        let hotkey_str_clone = app_config.reconnect_hotkey.clone();
        let modifier_keys_clone = modifier_keys_vec.clone();
        main_key_to_bind.bind(move || {
            let mut all_modifiers_pressed = true;
            if !modifier_keys_clone.is_empty() {
                for modifier in &modifier_keys_clone {
                    if !modifier.is_pressed() {
                        all_modifiers_pressed = false;
                        break;
                    }
                }
            }

            if all_modifiers_pressed {
                println!("全局热键 {} 被按下 (inputbot)", hotkey_str_clone);
            }
        });
        println!("已注册热键: {}", app_config.reconnect_hotkey);
    } else {
        println!(
            "警告: 无法从配置文件解析或注册主热键: {}。将不会注册热键。",
            app_config.reconnect_hotkey
        );
    }

    std::thread::spawn(|| {
        inputbot::handle_input_events();
    });

    std::thread::spawn(|| {
        window::app();
    });

    match rx.recv() {
        Ok(_) => println!("收到退出信号，程序即将关闭。"),
        Err(_) => println!("退出信号通道发生错误，程序即将关闭。"),
    }

    Ok(())
}
