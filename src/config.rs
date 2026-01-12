use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Constants {
    pub bepinex_url: String,
    pub user_agent: String,
    pub default_game_dir: String,
    pub app_title: String,
    pub log_max_height: f32,
    pub mods_max_height: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub constants: Constants,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from embedded Config.toml compiled into the binary
    pub fn load_embedded() -> Self {
        const CONFIG_TOML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Config.toml"));
        match toml::from_str::<Config>(CONFIG_TOML) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: Failed to parse embedded config: {}", e);
                eprintln!("Using built-in defaults.");
                Self::default()
            }
        }
    }

    pub fn load_or_default(path: &Path) -> Self {
        match Self::load(path) {
            Ok(cfg) => cfg,
            Err(_e) => {
                eprintln!("Config.toml not found; using embedded defaults.");
                Self::load_embedded()
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            constants: Constants {
                bepinex_url: "https://builds.bepinex.dev/projects/bepinex_be/752/BepInEx-Unity.IL2CPP-win-x64-6.0.0-be.752%2Bdd0655f.zip".to_string(),
                user_agent: "restaurats-mod-manager".to_string(),
                default_game_dir: r"C:\Program Files (x86)\Steam\steamapps\common\Restaurats".to_string(),
                app_title: "Restaurats Mod Manager".to_string(),
                log_max_height: 160.0,
                mods_max_height: 220.0,
            },
        }
    }
}
