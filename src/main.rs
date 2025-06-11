#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use crossbeam_channel::{select, unbounded};
use log::{error, info, warn};

mod config;
mod gui;
mod hearthstone;
mod hotkey;
mod logger;
mod network;
mod process;
mod tray;

const PROCESS_NAME: &str = "WXDrive_x64.exe";
const _LOGFILE_NAME: &str = "Hearthstone.log";

fn main() -> Result<()> {
    logger::init_logger()?;

    let app_config = config::get_config();
    info!(
        "加载的配置: {:?}",
        app_config
            .read()
            .map_err(|e| anyhow::anyhow!("无法获取配置读取锁: {}", e))?
    );

    // 托盘线程
    let (tray_tx, tray_rx) = unbounded::<tray::TrayMessage>();
    // GUI线程
    let (gui_tx, gui_rx) = unbounded::<gui::GuiMessage>();

    let reconnect_hotkey = app_config
        .read()
        .map_err(|e| anyhow::anyhow!("无法获取配置读取锁: {}", e))?
        .reconnect_hotkey
        .clone();
    let mut _tray_item = tray::setup_tray(tray_tx.clone(), &reconnect_hotkey)?;

    let reconnect_hotkey_clone = reconnect_hotkey.clone();
    let (main_key_opt, modifier_keys_vec) = hotkey::parse_hotkey_config(&reconnect_hotkey_clone);
    let mut current_registered_key: Option<inputbot::KeybdKey> = None;
    if let Some(main_key_to_bind) = main_key_opt {
        hotkey::register_hotkey(main_key_to_bind, modifier_keys_vec);
        current_registered_key = Some(main_key_to_bind);
    } else {
        warn!(
            "警告: 无法从配置文件解析或注册主热键: {}。将不会注册热键。",
            reconnect_hotkey_clone
        );
    }
    std::thread::spawn(move || {
        inputbot::handle_input_events();
    });
    std::thread::spawn(move || {
        hearthstone::watch_log().unwrap_or_else(|e| error!("日志监控线程发生错误: {}", e));
    });

    loop {
        select! {
            recv(tray_rx) -> msg => {
                match msg {
                    Ok(tray_msg) => {
                        info!("收到托盘消息: {:?}", tray_msg);
                        match tray_msg {
                            tray::TrayMessage::Exit => {
                                info!("收到退出消息，正在退出应用...");
                                break;
                            }
                            tray::TrayMessage::Reconnect => {
                                match hearthstone::reconnect(PROCESS_NAME) {
                                    Ok(_) => {
                                        info!("重连操作成功。");
                                    }
                                    Err(e) => {
                                        error!("重连操作失败: {}", e);
                                    }
                                }
                            }
                            tray::TrayMessage::Setting => {
                                let gui_tx_clone = gui_tx.clone();
                                let reconnect_hotkey = app_config.read().map_err(|e| anyhow::anyhow!("无法获取配置读取锁: {}", e))?.reconnect_hotkey.clone();
                                std::thread::spawn(move || {
                                    gui::app(gui_tx_clone, &reconnect_hotkey).unwrap();
                                });
                            }
                            tray::TrayMessage::UpdateMenu(config) => {
                                _tray_item = tray::setup_tray(tray_tx.clone(), &config)?;
                            }
                        }
                    }
                    Err(e) => error!("接收托盘消息失败: {}", e),
                }
            }
            recv(gui_rx) -> msg => {
                match msg {
                    Ok(gui_msg) => {
                        match gui_msg {
                            gui::GuiMessage::SaveHotKeys(reconnect_hotkey) => {
                                let mut config = app_config.write()
                                    .map_err(|e| anyhow::anyhow!("无法获取配置写入锁: {}", e))?;
                                config.reconnect_hotkey = reconnect_hotkey.clone();
                                config.save()?;
                                tray_tx.send(tray::TrayMessage::UpdateMenu(reconnect_hotkey.clone()))?;
                                let (main_key_opt, modifier_keys_vec) = hotkey::parse_hotkey_config(&reconnect_hotkey);
                                if let Some(main_key_to_bind) = main_key_opt {
                                    if let Some(current_key) = current_registered_key {
                                        info!("正在注销当前热键: {:?}", current_key);
                                        hotkey::unregister_hotkey(current_key);
                                    }
                                    hotkey::register_hotkey(main_key_to_bind, modifier_keys_vec);
                                    current_registered_key = Some(main_key_to_bind);
                                } else {
                                    warn!(
                                        "警告: 无法从配置文件解析或注册主热键: {}。将不会注册热键。",
                                        reconnect_hotkey_clone
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => error!("接收GUI消息失败: {}", e),
                }
            }
        }
    }

    Ok(())
}
