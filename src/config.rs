use std::fs;
use std::path::{Path, PathBuf};
use dirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_minecraft_path: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir().unwrap().join("storytime-launcher");
        let config_file = config_dir.join("config.toml");
        
        if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(_) => println!("󰅖 Ошибка чтения конфига, создаю новый"),
                },
                Err(_) => println!("󰅖 Ошибка чтения конфига, создаю новый"),
            }
        }
        
        Config {
            default_minecraft_path: None,
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().unwrap().join("storytime-launcher");
        fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("config.toml");
        let toml = toml::to_string(&self)?;
        fs::write(config_file, toml)?;
        
        Ok(())
    }
    
    pub fn get_default_path(&self) -> Option<PathBuf> {
        self.default_minecraft_path
            .as_ref()
            .map(|p| PathBuf::from(p))
    }
    
    pub fn set_default_path(&mut self, path: &Path) {
        self.default_minecraft_path = Some(path.display().to_string());
        if let Err(e) = self.save() {
            eprintln!("󰅖 Ошибка сохранения конфига: {}", e);
        }
    }
}