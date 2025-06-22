#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::AppContext;
use std::net::Ipv4Addr;

use anyhow::Result;
use crossbeam_channel::{select, unbounded};
use gpui::{
    App, Application, Bounds, SharedString, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use is_elevated::is_elevated;
use log::{error, info, warn};

use crate::gui::Setting;

mod config;
mod gui;
mod hearthstone;
mod hotkey;
mod logger;
mod network;
mod process;
mod tray;

const PROCESS_NAME: &str = "Hearthstone.exe";
const LOGFILE_NAME: &str = "Hearthstone.log";

fn main() -> Result<()> {
    logger::init_logger()?;

    if !is_elevated() {
        error!("应用未以管理员权限运行, 软件无法正常工作。");
        return Err(anyhow::anyhow!(
            "应用未以管理员权限运行, 软件无法正常工作。"
        ));
    }

    let mut hs_ip: Option<Ipv4Addr> = None;
    let mut hs_port: Option<u16> = None;

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
    // 日志监控线程
    let (log_tx, log_rx) = unbounded::<hearthstone::LogMessage>();

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
        hotkey::register_hotkey(tray_tx.clone(), main_key_to_bind, modifier_keys_vec);
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
    let log_tx_clone = log_tx.clone();
    std::thread::spawn(move || {
        loop {
            match hearthstone::watch_log(log_tx_clone.clone()) {
                Ok(_) => {
                    info!("日志监控线程正常退出");
                    break;
                }
                Err(e) => {
                    error!("日志监控线程意外退出。错误: {} 5秒后重新启动...", e);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }
    });

    std::thread::spawn(move || -> Result<()> {
        loop {
            select! {
                recv(tray_rx) -> msg => {
                    match msg {
                        Ok(tray_msg) => {
                            info!("收到托盘消息: {:?}", tray_msg);
                            match tray_msg {
                                tray::TrayMessage::Exit => {
                                    info!("收到退出消息，正在退出应用...");
                                    return Ok(());
                                }
                                tray::TrayMessage::Reconnect => {
                                    match hearthstone::reconnect(hs_ip, hs_port) {
                                        Ok(_) => {
                                            info!("重连操作成功。");
                                        }
                                        Err(e) => {
                                            error!("重连操作失败: {}", e);
                                        }
                                    }
                                }
                                tray::TrayMessage::Setting => {
                                    // TODO 显示窗口
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
                                        hotkey::register_hotkey(tray_tx.clone(),main_key_to_bind, modifier_keys_vec);
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
                recv(log_rx) -> msg => {
                    match msg {
                        Ok(log_msg) => {
                            info!("检测到炉石IP变化: {:?}", log_msg);
                            hs_ip = log_msg.ip;
                            hs_port = Some(log_msg.port);
                        }
                        Err(e) => error!("接收日志消息失败: {}", e),
                    }
                }
            }
        }
    });

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(600.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("快捷键设置".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    window.on_window_should_close(cx, |_, _| false);
                    cx.new(|_| Setting {
                        hotkeys: reconnect_hotkey.into(),
                        current_pressed_keys: SharedString::from(""),
                        tx: gui_tx,
                    })
                },
            )
            .unwrap();
        let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
        cx.observe_keystrokes(move |event, _window, cx| {
            let keystroke: &gpui::Keystroke = &event.keystroke;
            // 构建按键字符串
            let mut key_parts = Vec::new();
            // 添加修饰键
            if keystroke.modifiers.control {
                key_parts.push("Ctrl".to_string());
            }
            if keystroke.modifiers.alt {
                key_parts.push("Alt".to_string());
            }
            if keystroke.modifiers.shift {
                key_parts.push("Shift".to_string());
            }
            if keystroke.modifiers.platform {
                key_parts.push("Win".to_string());
            }
            // 添加主键
            let main_key = match keystroke.key.as_str() {
                "escape" => "Esc",
                "enter" => "Enter",
                "space" => "Space",
                "tab" => "Tab",
                key => key,
            };
            key_parts.push(main_key.to_uppercase());
            let pressed_keys = if key_parts.len() > 1 {
                key_parts.join("+")
            } else {
                key_parts.join("")
            };
            view.update(cx, |view, cx| {
                view.current_pressed_keys = SharedString::from(pressed_keys.clone());
                cx.notify();
            })
        })
        .detach();
    });
    Ok(())
}
