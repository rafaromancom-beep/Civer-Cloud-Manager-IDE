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
                    name: "Antigravity 2.0 (v2.6.0 - Principal)".to_string(),
                    description: "Versión principal y motor actual de Antigravity (Antigravity-v2.6.0-x64.exe).".to_string(),
                    download_url: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\Instaladores-Oficiales\\Antigravity-v2.6.0-x64.exe".to_string()),
                    local_path: Some("C:\\Users\\Administrator\\AppData\\Local\\Programs\\Antigravity\\Antigravity.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "antigravity_normal".to_string(),
                    name: "Antigravity IDE (v2.1.1 - Legacy)".to_string(),
                    description: "Versión legacy de Antigravity IDE (Antigravity-IDE-v2.1.1.exe).".to_string(),
                    download_url: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\Instaladores-Oficiales\\Antigravity-IDE-v2.1.1.exe".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\AntigravityIDE-v2.1.1\\Antigravity.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "antigravity_cli".to_string(),
                    name: "Antigravity CLI (v1.1.11)".to_string(),
                    description: "Interfaz de línea de comandos para Antigravity en terminal.".to_string(),
                    download_url: Some("https://antigravity.google/cli/install.ps1".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\AntigravityIDE\\agy.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "antigravity_sdk".to_string(),
                    name: "Antigravity SDK (v0.1.10)".to_string(),
                    description: "Kit de desarrollo y herramientas de extensión de Antigravity.".to_string(),
                    download_url: Some("https://github.com/google-antigravity/antigravity-sdk-python".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\Antigravity-Agent\\antigravity-agent.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "rclone".to_string(),
                    name: "Rclone".to_string(),
                    description: "Sincronizador de archivos para el ecosistema Civer Cloud.".to_string(),
                    download_url: Some("https://downloads.rclone.org/v1.68.2/rclone-v1.68.2-windows-amd64.zip".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\BajoNivel\\rclone.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "kopia".to_string(),
                    name: "Kopia Backup".to_string(),
                    description: "Herramienta de encriptación y deduplicación para respaldos pesados.".to_string(),
                    download_url: Some("https://github.com/kopia/kopia/releases/download/v0.17.0/kopia-0.17.0-windows-x64.zip".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\KopiaConfig\\kopia.exe".to_string()),
                },

                BootstrapperProgram {
                    id: "tailscale".to_string(),
                    name: "Tailscale".to_string(),
                    description: "Red Mesh P2P. Logueado silenciosamente en todos los nodos.".to_string(),
                    download_url: Some("https://pkgs.tailscale.com/stable/tailscale-setup-latest.exe".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\Tailscale\\tailscale.exe".to_string()),
                },
                BootstrapperProgram {
                    id: "protonvpn".to_string(),
                    name: "Proton VPN".to_string(),
                    description: "Túnel cifrado para evadir censura o firewalls locales.".to_string(),
                    download_url: Some("https://protonvpn.com/download/ProtonVPN_win_v3.2.10.exe".to_string()),
                    local_path: Some("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\ProtonVPN\\Binaries\\ProtonVPN.Launcher.exe".to_string()),
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
    let default_config = BootstrapperConfig::default();
    
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(mut loaded_config) = serde_json::from_str::<BootstrapperConfig>(&contents) {
                // Purge autogravity if present in saved file
                let mut changed = false;
                if loaded_config.programs.iter().any(|p| p.id == "autogravity") {
                    loaded_config.programs.retain(|p| p.id != "autogravity");
                    changed = true;
                }
                // Merge missing programs from default into loaded
                for default_prog in default_config.programs.clone() {
                    if !loaded_config.programs.iter().any(|p| p.id == default_prog.id) {
                        loaded_config.programs.push(default_prog);
                        changed = true;
                    }
                }
                if changed {
                    let _ = save_config(&loaded_config);
                }
                return loaded_config;
            }
        }
    }
    // Return default and save it if not exists
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
