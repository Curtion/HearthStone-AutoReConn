#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};
use tray_item::IconSource;

use flume::{Selector, unbounded};
use is_elevated::is_elevated;
use log::{error, info, warn};

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

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    logger::init_logger()?;

    if !is_elevated() {
        error!("应用未以管理员权限运行, 软件无法正常工作。");
        return Err(anyhow::anyhow!(
            "应用未以管理员权限运行, 软件无法正常工作。"
        ));
    }

    let hs_ip: Arc<Mutex<Option<Ipv4Addr>>> = Arc::new(Mutex::new(None));
    let hs_port: Arc<Mutex<Option<u16>>> = Arc::new(Mutex::new(None));
    let is_disconnecting = Arc::new(Mutex::new(false));

    let app_config = config::get_config();
    info!(
        "加载的配置: {:?}",
        app_config
            .read()
            .map_err(|e| anyhow::anyhow!("无法获取配置读取锁: {}", e))?
    );

    // 托盘线程
    let (tray_tx, tray_rx) = unbounded::<tray::TrayMessage>();
    // GUI线程(GUI端发送消息)
    let (gui_out_tx, gui_out_rx) = unbounded::<gui::GuiOutMessage>();
    // GUI线程(GUI端接收消息)
    let (gui_in_tx, gui_in_rx) = unbounded::<gui::GuiInMessage>();
    // 日志监控线程
    let (log_tx, log_rx) = unbounded::<hearthstone::LogMessage>();

    let reconnect_hotkey = app_config
        .read()
        .map_err(|e| anyhow::anyhow!("无法获取配置读取锁: {}", e))?
        .reconnect_hotkey
        .clone();
    let tray_item = Arc::new(Mutex::new(tray::setup_tray(
        tray_tx.clone(),
        &reconnect_hotkey,
    )?));

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

    let hs_ip_clone = Arc::clone(&hs_ip);
    let hs_port_clone = Arc::clone(&hs_port);
    let gui_in_tx_clone = gui_in_tx.clone();
    let is_disconnecting_clone = Arc::clone(&is_disconnecting);
    let tray_item_clone = Arc::clone(&tray_item);
    std::thread::spawn(move || -> anyhow::Result<()> {
        loop {
            let selector_res = Selector::new()
                .recv(&tray_rx, |msg| -> anyhow::Result<()> {
                    match msg {
                        Ok(tray_msg) => {
                            info!("收到托盘消息: {:?}", tray_msg);
                            match tray_msg {
                                tray::TrayMessage::Exit => {
                                    gui_in_tx_clone
                                        .send(gui::GuiInMessage::Exit)
                                        .map_err(|e| anyhow::anyhow!("无法发送GUI消息: {}", e))?;
                                    return Err(anyhow::anyhow!("Exit signal received"));
                                }
                                tray::TrayMessage::Reconnect => {
                                    let hs_ip = hs_ip_clone
                                        .lock()
                                        .map_err(|e| anyhow::anyhow!("无法获取炉石IP锁: {}", e))?;
                                    let hs_port = hs_port_clone
                                        .lock()
                                        .map_err(|e| anyhow::anyhow!("无法获取炉石端口锁: {}", e))?;
                                    match hearthstone::reconnect(*hs_ip, *hs_port) {
                                        Ok(_) => {
                                            info!("重连操作成功。");
                                            let mut is_disconnecting = is_disconnecting_clone
                                                .lock()
                                                .map_err(|e| anyhow::anyhow!("无法获取断开连接状态锁: {}", e))?;
                                            *is_disconnecting = true;
                                            let mut tray = tray_item_clone
                                                .lock()
                                                .map_err(|e| anyhow::anyhow!("无法获取托盘项目锁: {}", e))?;
                                            tray.set_icon(IconSource::Resource("#3"))?;
                                        }
                                        Err(e) => {
                                            error!("重连操作失败: {}", e);
                                        }
                                    }
                                }
                                tray::TrayMessage::Setting => {
                                    gui_in_tx_clone
                                        .send(gui::GuiInMessage::Show)
                                        .map_err(|e| anyhow::anyhow!("无法发送GUI消息: {}", e))?;
                                }
                                tray::TrayMessage::UpdateMenu(config) => {
                                    let mut tray = tray_item_clone
                                        .lock()
                                        .map_err(|e| anyhow::anyhow!("无法获取托盘项目锁: {}", e))?;
                                    *tray = tray::setup_tray(tray_tx.clone(), &config)?;
                                }
                            }
                        }
                        Err(e) => error!("接收托盘消息失败: {}", e),
                    }
                    Ok(())
                })
                .recv(&gui_out_rx, |msg| -> anyhow::Result<()> {
                    match msg {
                        Ok(gui_msg) => match gui_msg {
                            gui::GuiOutMessage::SaveHotKeys(reconnect_hotkey) => {
                                let mut config = app_config
                                    .write()
                                    .map_err(|e| anyhow::anyhow!("无法获取配置写入锁: {}", e))?;
                                config.reconnect_hotkey = reconnect_hotkey.clone();
                                config.save()?;
                                tray_tx.send(tray::TrayMessage::UpdateMenu(
                                    reconnect_hotkey.clone(),
                                ))?;
                                let (main_key_opt, modifier_keys_vec) =
                                    hotkey::parse_hotkey_config(&reconnect_hotkey);
                                if let Some(main_key_to_bind) = main_key_opt {
                                    if let Some(current_key) = current_registered_key {
                                        info!("正在注销当前热键: {:?}", current_key);
                                        hotkey::unregister_hotkey(current_key);
                                    }
                                    hotkey::register_hotkey(
                                        tray_tx.clone(),
                                        main_key_to_bind,
                                        modifier_keys_vec,
                                    );
                                    current_registered_key = Some(main_key_to_bind);
                                } else {
                                    warn!(
                                        "警告: 无法从配置文件解析或注册主热键: {}。将不会注册热键。",
                                        reconnect_hotkey_clone
                                    );
                                }
                            }
                        },
                        Err(e) => error!("接收GUI消息失败: {}", e),
                    }
                    Ok(())
                })
                .recv(&log_rx, |msg| -> anyhow::Result<()> {
                    match msg {
                        Ok(log_msg) => {
                            info!("检测到炉石IP变化: {:?}", log_msg);
                            *hs_ip_clone.lock().unwrap() = log_msg.ip;
                            *hs_port_clone.lock().unwrap() = Some(log_msg.port);
                            let mut is_disconnecting = is_disconnecting_clone
                                .lock()
                                .map_err(|e| anyhow::anyhow!("无法获取断开连接状态锁: {}", e))?;
                            if *is_disconnecting {
                                *is_disconnecting = false;
                                let mut tray = tray_item_clone
                                    .lock()
                                    .map_err(|e| anyhow::anyhow!("无法获取托盘项目锁: {}", e))?;
                                tray.set_icon(IconSource::Resource("#1"))?;
                                info!("已恢复默认托盘图标。");
                            }
                        }
                        Err(e) => error!("接收日志消息失败: {}", e),
                    }
                    Ok(())
                })
                .wait();

            if let Err(e) = selector_res {
                if e.to_string() == "Exit signal received" {
                    info!("正在中断消息循环...");
                    break;
                }
                error!("选择器错误: {}", e);
            }
        }
        Ok(())
    });

    let reconnect_hotkey_clone = reconnect_hotkey.clone();

    let main_window = MainWindow::new()?;
    let main_window_weak = main_window.as_weak();
    main_window.set_hotkeys(reconnect_hotkey_clone.into());
    main_window.on_save_hotkeys(move |value| {
        let value_clone = value.clone();
        if let Ok(_) = gui_out_tx.send(gui::GuiOutMessage::SaveHotKeys(value.into())) {
            if let Some(main_window) = main_window_weak.upgrade() {
                main_window.set_hotkeys(value_clone.into());
            }
        }
    });

    main_window.window().on_close_requested(move || {
        info!("主窗口请求关闭，正在处理...");
        slint::CloseRequestResponse::HideWindow
    });

    let main_window_weak = main_window.as_weak();
    std::thread::spawn(move || {
        for message in &gui_in_rx {
            match message {
                gui::GuiInMessage::Exit => {
                    let _ = slint::quit_event_loop();
                    break;
                }
                gui::GuiInMessage::Show => {
                    let window_weak_clone = main_window_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(main_window) = window_weak_clone.upgrade() {
                            let _ = main_window.show();
                        } else {
                            error!("无法显示窗口, 似乎窗口已被销毁");
                        }
                    })
                    .unwrap();
                }
            }
        }
    });

    main_window.show()?;
    slint::run_event_loop_until_quit()?;
    info!("应用正在退出...");
    Ok(())
}
