#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use crossbeam_channel::{select, unbounded};
use log::{error, info, warn};

mod config;
mod hearthstone;
mod hotkey;
mod logger;
mod network;
mod process;
mod tray;
mod window;

const PROCESS_NAME: &str = "Hearthstone.exe";
const LOGFILE_NAME: &str = "Hearthstone.log";

fn main() -> Result<()> {
    logger::init_logger()?;
    // hearthstone::watch_log()?;
    // let data = process::get_process_by_name(PROCESS_NAME)?;
    // info!("获取到的进程信息: {:#?}", data);
    // let data = network::get_process_by_pid(data.get(0).map_or(0, |p| p.pid))?;
    // info!("获取到的网络信息: {:#?}", data);

    let app_config = config::load_config();
    info!("加载的配置: {:?}", app_config);

    // 托盘线程
    let (tx, rx) = unbounded::<tray::TrayMessage>();

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

    loop {
        select! {
            recv(rx) -> msg => {
                match msg {
                    Ok(tray_msg) => {
                        info!("收到托盘消息: {:?}", tray_msg);
                        match tray_msg {
                            tray::TrayMessage::Exit => {
                                info!("收到退出消息，正在退出应用...");
                                break;
                            }
                            tray::TrayMessage::Reconnect => {
                                info!("执行重连操作...");
                            }
                            tray::TrayMessage::Setting => {
                                std::thread::spawn(|| {
                                    window::app();
                                });
                            }
                        }
                    }
                    Err(e) => error!("接收托盘消息失败: {}", e),
                }
            }
        }
    }

    Ok(())
}
