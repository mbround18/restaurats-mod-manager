use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModEntry {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub source_zip: Option<String>,
    pub installed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModIndex {
    pub mods: Vec<ModEntry>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    GettingStarted,
    Mods,
}

pub struct AppState {
    pub game_dir: PathBuf,
    pub bep_status: String,
    pub mods: ModIndex,
    pub status_log: Vec<String>,
    pub custom_bep_url: String,
    pub is_busy: bool,
    pub logo_texture: Option<Box<dyn std::any::Any>>,
    pub current_tab: Tab,
    pub bep_ready: bool,
    pub poller_flag: Option<Arc<Mutex<bool>>>,
    pub install_task: Option<Arc<Mutex<Option<Result<(), String>>>>>,
    pub config: Config,
}

impl Default for AppState {
    fn default() -> Self {
        let default_path =
            PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Restaurats");
        Self {
            game_dir: default_path,
            bep_status: String::new(),
            mods: ModIndex::default(),
            status_log: Vec::new(),
            custom_bep_url: String::new(),
            is_busy: false,
            logo_texture: None,
            current_tab: Tab::GettingStarted,
            bep_ready: false,
            poller_flag: None,
            install_task: None,
            config: Config::default(),
        }
    }
}
