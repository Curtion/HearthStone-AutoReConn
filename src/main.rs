#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 引入必要的模块
use windows::Win32::Foundation::HWND;
// GetActiveWindow 现在位于 KeyboardAndMouse 模块下
use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
// ShowWindow 仍然在 WindowsAndMessaging 模块下
use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindow};

use gpui::AppContext;
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

use flume::{Selector, unbounded};
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

    let hs_ip_clone = Arc::clone(&hs_ip);
    let hs_port_clone = Arc::clone(&hs_port);
    let gui_in_tx_clone = gui_in_tx.clone();
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
                                    _tray_item = tray::setup_tray(tray_tx.clone(), &config)?;
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
                            *hs_ip.lock().unwrap() = log_msg.ip;
                            *hs_port.lock().unwrap() = Some(log_msg.port);
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
                    show: false,
                    ..Default::default()
                },
                |window, cx| {
                    window.on_window_should_close(cx, move |_, _| {
                        gui_in_tx.send(gui::GuiInMessage::Hide).unwrap_or_else(|e| {
                            error!("无法发送退出消息: {}", e);
                        });
                        false
                    });
                    cx.new(|_| Setting {
                        hotkeys: reconnect_hotkey.into(),
                        current_pressed_keys: SharedString::from(""),
                        tx: gui_out_tx,
                        window_handle: None,
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
        let window_clone = window.clone();
        cx.spawn(async move |cx| -> anyhow::Result<()> {
            while let Ok(gui_msg) = gui_in_rx.recv_async().await {
                info!("GUI收到消息: {:?}", gui_msg);
                let result: anyhow::Result<()> =
                    window_clone.update(cx, |view, window, cx| match gui_msg {
                        gui::GuiInMessage::Exit => {
                            info!("退出应用...");
                            cx.quit();
                        }
                        gui::GuiInMessage::Show => {
                            if let Some(hwnd) = view.window_handle {
                                if hwnd.0 != std::ptr::null_mut() {
                                    show_window_by_handle(hwnd);
                                }
                            } else {
                                window.activate_window();
                            }
                        }
                        gui::GuiInMessage::Hide => {
                            let window = hide_current_window();
                            view.window_handle = window;
                        }
                    });

                if let Err(e) = result {
                    error!("处理GUI消息时出错: {}", e);
                }
                if let gui::GuiInMessage::Exit = gui_msg {
                    break;
                }
            }
            Ok(())
        })
        .detach();
    });
    Ok(())
}

// gpui未实现隐藏功能: https://github.com/zed-industries/zed/blob/f3896a2d51028514744df4a903e7cd3f5bbd0224/crates/gpui/src/platform/windows/platform.rs#L399
fn hide_current_window() -> Option<HWND> {
    unsafe {
        let hwnd: HWND = GetActiveWindow();
        if hwnd.0 != std::ptr::null_mut() {
            let _ = ShowWindow(hwnd, SW_HIDE);
            Some(hwnd)
        } else {
            error!("无法获取当前活动窗口句柄");
            None
        }
    }
}

fn show_window_by_handle(hwnd: HWND) {
    unsafe {
        if hwnd.0 != std::ptr::null_mut() {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }
}
