#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use crossbeam_channel::bounded;
use log::{error, info, warn};
use simplelog::*;
use std::fs::File;

mod config;
mod hotkey;
mod tray;
mod window;

fn init_logger() -> Result<()> {
    // 获取exe所在目录
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let log_file_path = exe_dir.join("hsarec.log");

    // 创建日志文件
    let log_file = File::create(&log_file_path)?;

    let config = ConfigBuilder::new()
        .add_filter_allow_str("hsarec")
        .set_target_level(LevelFilter::Info)
        .set_location_level(LevelFilter::Off)
        .build();

    // 配置日志输出到文件和控制台
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, config, log_file),
    ])?;

    info!("日志系统已初始化，日志文件: {:?}", log_file_path);
    Ok(())
}

fn main() -> Result<()> {
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
