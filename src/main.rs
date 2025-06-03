#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use crossbeam_channel::bounded;
use log::{info, warn, error};
use std::path::Path;

mod config;
mod hotkey;
mod tray;
mod window;

/// 初始化日志系统，将日志保存到exe目录的log文件中
fn init_logger() -> Result<()> {
    // 获取当前exe路径
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
    let log_file_path = exe_dir.join("hsarec.log");
    
    // 使用fern配置日志，同时输出到控制台和文件
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout()) // 输出到控制台
        .chain(fern::log_file(&log_file_path)?) // 输出到文件
        .apply()?;
    
    info!("日志系统已初始化，日志文件路径: {:?}", log_file_path);
    Ok(())
}

fn main() -> Result<()> {
    // 初始化日志系统
    init_logger()?;
    
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
            }            if all_modifiers_pressed {
                info!("全局热键 {} 被按下 (inputbot)", hotkey_str_clone);
            }        });
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
    });    match rx.recv() {
        Ok(_) => info!("收到退出信号，程序即将关闭。"),
        Err(_) => error!("退出信号通道发生错误，程序即将关闭。"),
    }

    Ok(())
}
