use anyhow::Result;
use log::info;
use simplelog::*;
use std::fs::OpenOptions;

pub fn init_logger() -> Result<()> {
    // 获取exe所在目录
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let log_file_path = exe_dir.join("hsarec.log");

    // 以追加模式打开日志文件
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    let config = ConfigBuilder::new()
        .add_filter_allow_str("hsarec")
        .set_target_level(LevelFilter::Info)
        .set_location_level(LevelFilter::Off)
        .set_time_offset_to_local()
        .unwrap_or_else(|builder| builder)
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
