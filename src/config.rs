use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use etcetera::{AppStrategy, AppStrategyArgs, app_strategy::choose_native_strategy};
use freya::prelude::{State, use_consume};
use serde::{Deserialize, Serialize};

use crate::{Session, ThemeVariant, default_theme_source};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ThemeScheme {
    Light,
    #[default]
    Dark,
}

impl ThemeScheme {
    pub fn toggle(&mut self) {
        match self {
            ThemeScheme::Light => *self = ThemeScheme::Dark,
            ThemeScheme::Dark => *self = ThemeScheme::Light,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ThemeConfig {
    #[serde(default)]
    pub scheme: ThemeScheme,
    #[serde(default)]
    pub variant: ThemeVariant,
    #[serde(default = "default_theme_source")]
    pub theme_source: u32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            scheme: Default::default(),
            variant: Default::default(),
            theme_source: default_theme_source(),
        }
    }
}

fn default_api_url() -> String {
    "https://api.stoat.chat".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    #[serde(default = "default_api_url")]
    pub api: String,
    #[serde(default)]
    pub session: Option<Session>,
    #[serde(default)]
    pub last_channels: HashMap<String, String>,
    #[serde(default)]
    pub collapsed_categories: HashSet<String>,
    #[serde(default)]
    pub hide_channel_list: bool,
    #[serde(default)]
    pub hide_members_list: bool,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub drafts: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api: default_api_url(),
            session: Default::default(),
            last_channels: Default::default(),
            collapsed_categories: Default::default(),
            hide_channel_list: Default::default(),
            hide_members_list: Default::default(),
            theme: Default::default(),
            drafts: Default::default(),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let strategy = choose_native_strategy(AppStrategyArgs {
        top_level_domain: "live".to_string(),
        author: "zomatree".to_string(),
        app_name: "Ermine".to_string(),
    })
    .unwrap();

    let mut dir = strategy.config_dir();
    let _ = std::fs::create_dir_all(&dir);

    dir.push("config.json");

    log::debug!("Config file: {dir:?}");

    dir
}

pub fn read_config() -> Config {
    let path = get_config_path();

    if let Ok(value) = std::fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str(&value)
    {
        config
    } else {
        Config::default()
    }
}

pub fn write_config(config: &Config) {
    let path = get_config_path();

    std::fs::write(path, serde_json::to_string(config).unwrap()).unwrap();
}

pub fn use_config() -> State<Config> {
    use_consume()
}
