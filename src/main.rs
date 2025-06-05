#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use crossbeam_channel::bounded;
use log::{error, info, warn};

mod config;
mod hotkey;
mod logger;
mod network;
mod tray;
mod window;
mod process;

const PROCESS_NAME: &str = "Hearthstone.exe";

fn main() -> Result<()> {
    logger::init_logger()?;

    let data = process::get_process_by_name(PROCESS_NAME)?;

    info!("获取到的进程信息: {:#?}", data);

    let data = network::get_process_by_pid(data.get(0).map_or(0, |p| p.pid))?;

    info!("获取到的网络信息: {:#?}", data);

    let app_config = config::load_config();
    info!("加载的配置: {:?}", app_config);

    // 用于托盘图标退出事件
    let (tx, rx) = bounded::<()>(1);

    let _tray_item = tray::setup_tray(tx)?;

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
                info!("全局热键 {} 被按下 (inputbot)", hotkey_str_clone);
            }
        });
        info!("已注册热键: {}", app_config.reconnect_hotkey);
    } else {
        warn!(
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
        Ok(_) => info!("收到退出信号，程序即将关闭。"),
        Err(_) => error!("退出信号通道发生错误，程序即将关闭。"),
    }

    Ok(())
}
