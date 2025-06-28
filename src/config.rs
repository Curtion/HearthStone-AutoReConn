use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub reconnect_hotkey: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            reconnect_hotkey: "Shift+Alt+R".to_string(),
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn load() -> Self {
        let config_path = match Self::get_config_path() {
            Ok(path) => path,
            Err(msg) => {
                warn!("警告: {}，将使用默认配置。", msg);
                return Self::default();
            }
        };

        let config = if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            warn!("警告: 配置文件 {:?} 不存在。将使用默认配置。", config_path);
            let default_config = Self::default();
            if let Err(e) = default_config.save_to_file(&config_path) {
                warn!("警告: 无法创建默认配置文件: {}", e);
            } else {
                info!("已在 {:?} 创建默认配置文件。", config_path);
            }
            default_config
        };

        config
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        self.save_to_file(&config_path)
    }

    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf> {
        let current_exe = std::env::current_exe()?;
        let parent_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取当前可执行文件的父目录"))?;
        Ok(parent_dir.join("config.toml"))
    }

    /// 从指定文件加载配置
    fn load_from_file(config_path: &PathBuf) -> Self {
        match fs::read_to_string(config_path) {
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
                    Self::default()
                }
            },
            Err(e) => {
                warn!(
                    "警告: 读取配置文件 {:?} 失败: {}。将使用默认配置。",
                    config_path, e
                );
                Self::default()
            }
        }
    }

    /// 保存配置到指定文件
    fn save_to_file(&self, config_path: &PathBuf) -> Result<()> {
        let config_content = toml::to_string_pretty(self)?;
        fs::write(config_path, config_content)?;
        info!("配置已保存到: {:?}", config_path);
        Ok(())
    }
}

static CONFIG: OnceLock<Arc<RwLock<Config>>> = OnceLock::new();

/// 获取全局配置实例
pub fn get_config() -> Arc<RwLock<Config>> {
    CONFIG
        .get_or_init(|| {
            let config = Config::load();
            Arc::new(RwLock::new(config))
        })
        .clone()
}
