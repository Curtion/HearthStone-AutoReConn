use anyhow::Result;
use log::info;
use simplelog::*;
use std::fs::File;

pub fn init_logger() -> Result<()> {
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