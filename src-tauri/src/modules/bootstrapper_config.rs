use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::modules::logger;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BootstrapperProgram {
    pub id: String,
    pub name: String,
    pub description: String,
    pub download_url: Option<String>,
    pub local_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BootstrapperConfig {
    pub programs: Vec<BootstrapperProgram>,
}

impl Default for BootstrapperConfig {
    fn default() -> Self {
        Self {
            programs: vec![
                BootstrapperProgram {
                    id: "antigravity_classic".to_string(),
                    name: "Antigravity Clásico (2.0)".to_string(),
                    description: "El IDE legacy de Antigravity, necesario para proyectos anteriores.".to_string(),
                    download_url: Some("https://example.com/antigravity2.exe".to_string()),
                    local_path: Some("C:\\Users\\Usuario\\AppData\\Local\\Programs\\Antigravity\\Antigravity.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "antigravity_normal".to_string(),
                    name: "Antigravity Normal".to_string(),
                    description: "Versión estable principal de Antigravity.".to_string(),
                    download_url: Some("https://example.com/antigravity.exe".to_string()),
                    local_path: Some("C:\\Program Files\\Antigravity\\antigravity.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "antigravity_sdk".to_string(),
                    name: "Antigravity SDK".to_string(),
                    description: "Herramientas de desarrollo de Antigravity.".to_string(),
                    download_url: Some("https://example.com/sdk.zip".to_string()),
                    local_path: Some("C:\\AntigravitySDK\\bin\\sdk.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "rclone".to_string(),
                    name: "Rclone".to_string(),
                    description: "Sincronizador de archivos para el ecosistema Civer Cloud.".to_string(),
                    download_url: Some("https://downloads.rclone.org/v1.68.2/rclone-v1.68.2-windows-amd64.zip".to_string()),
                    local_path: Some("C:\\rclone\\rclone.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "kopia".to_string(),
                    name: "Kopia Backup".to_string(),
                    description: "Herramienta de encriptación y deduplicación para respaldos pesados.".to_string(),
                    download_url: Some("https://github.com/kopia/kopia/releases/download/v0.17.0/kopia-0.17.0-windows-x64.zip".to_string()),
                    local_path: Some("C:\\kopia\\kopia.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "autogravity".to_string(),
                    name: "AutoGravity".to_string(),
                    description: "Automatización de IA satelital de la familia Antigravity.".to_string(),
                    download_url: Some("https://example.com/autogravity.exe".to_string()),
                    local_path: Some("C:\\AutoGravity\\autogravity.exe".to_string()),
                }
            ],
        }
    }
}

pub fn get_config_path() -> PathBuf {
    // Usaremos la bóveda o un directorio estático
    let vault_path = Path::new("C:\\ProyectoCiverCloudUnificado");
    vault_path.join("bootstrapper_config.json")
}

pub fn load_config() -> BootstrapperConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<BootstrapperConfig>(&contents) {
                return config;
            }
        }
    }
    // Return default and save it if not exists
    let default_config = BootstrapperConfig::default();
    let _ = save_config(&default_config);
    default_config
}

pub fn save_config(config: &BootstrapperConfig) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    logger::log_info(&format!("Bootstrapper config saved to {:?}", path));
    Ok(())
}
