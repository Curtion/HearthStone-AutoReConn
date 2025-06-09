use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub reconnect_hotkey: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            reconnect_hotkey: "Ctrl+Shift+F12".to_string(),
        }
    }
}

pub fn load_config() -> Config {
    let config_path = if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent_dir) = current_exe.parent() {
            parent_dir.join("config.toml")
        } else {
            warn!("警告: 无法获取可执行文件的父目录，将使用默认配置。");
            return Config::default();
        }
    } else {
        warn!("警告: 无法获取可执行文件路径，将使用默认配置。");
        return Config::default();
    };

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    info!("成功加载配置文件: {:?}", config_path);
                    config
                }
                Err(e) => {
                    warn!(
                        "警告: 解析配置文件 {:?} 失败: {}。将使用默认配置。",
                        config_path, e
                    );
                    Config::default()
                }
            },
            Err(e) => {
                warn!(
                    "警告: 读取配置文件 {:?} 失败: {}。将使用默认配置。",
                    config_path, e
                );
                Config::default()
            }
        }
    } else {
        warn!("警告: 配置文件 {:?} 不存在。将使用默认配置。", config_path);
        // 尝试创建默认配置文件
        let default_config_content =
            toml::to_string_pretty(&Config::default()).unwrap_or_else(|e| {
                warn!("警告: 无法序列化默认配置: {}", e);
                String::new()
            });
        if !default_config_content.is_empty() {
            if let Err(e) = fs::write(&config_path, default_config_content) {
                warn!("警告: 无法创建默认配置文件 {:?}: {}", config_path, e);
            } else {
                info!("已在 {:?} 创建默认配置文件。", config_path);
            }
        }
        Config::default()
    }
}
