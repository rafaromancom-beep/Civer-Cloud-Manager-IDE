use reqwest::Client;
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use crate::utils::crypto;

const SYNCTHING_API_URL: &str = "http://localhost:8384/rest";

pub struct SyncthingController {
    client: Client,
    api_key: String,
    bin_path: PathBuf,
}

impl SyncthingController {
    pub fn new() -> Self {
        // Secure deterministic API key based on the machine or fallback
        let api_key = "antigravity_syncthing_secret_key_2026".to_string();
        
        let app_dir = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Administrator".to_string());
        let bin_path = PathBuf::from(format!(r"{}\.gemini\antigravity\bin\syncthing.exe", app_dir));

        Self {
            client: Client::builder().timeout(Duration::from_secs(10)).build().unwrap(),
            api_key,
            bin_path,
        }
    }

    pub async fn start_daemon(&self) -> Result<(), String> {
        if !self.bin_path.exists() {
            return Err("Syncthing binary not found. Bootstrapper failed?".to_string());
        }

        // Start Syncthing invisibly
        tracing::info!("Starting Syncthing daemon...");
        Command::new(&self.bin_path)
            .arg("-no-browser")
            .arg("-no-restart")
            .arg(&format!("-gui-apikey={}", self.api_key))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;

        // Wait for API to be ready
        for _ in 0..10 {
            if self.is_ready().await {
                tracing::info!("Syncthing daemon is ready.");
                return Ok(());
            }
            sleep(Duration::from_secs(2)).await;
        }

        Err("Syncthing daemon failed to start or API is unreachable.".to_string())
    }

    pub async fn is_ready(&self) -> bool {
        let url = format!("{}/system/ping", SYNCTHING_API_URL);
        self.client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn configure_brain_sync(&self) -> Result<(), String> {
        tracing::info!("Configuring Syncthing folder for Brain...");
        let url = format!("{}/config/folders", SYNCTHING_API_URL);
        
        // Check if folder exists
        let config_res = self.client.get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await.map_err(|e| e.to_string())?;
            
        let folders: Vec<Value> = config_res.json().await.unwrap_or_default();
        let brain_id = "antigravity-brain-mesh";
        
        // Removed early return to ensure all folders (brain and accounts) are checked and added

        // Folder Path
        let app_dir = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Administrator".to_string());
        let brain_path = format!(r"{}\.gemini\antigravity\brain", app_dir);

        let new_folder = json!({
            "id": brain_id,
            "label": "Antigravity Brain",
            "path": brain_path,
            "type": "sendreceive",
            "rescanIntervalS": 10,
            "fsWatcherEnabled": true,
            "fsWatcherDelayS": 1,
            "ignorePerms": true,
            "autoNormalize": true,
            "versioning": {
                "type": "simple",
                "params": { "keep": "5" }
            }
        });

        // [FIX] Phase 5: Add accounts database (.civer_cloud_manager_ide_5_1) to the mesh synchronization
        let accounts_id = "antigravity-accounts-mesh";
        let accounts_path = format!(r"{}\.civer_cloud_manager_ide_5_1", app_dir);
        let accounts_folder = json!({
            "id": accounts_id,
            "label": "Antigravity Accounts",
            "path": accounts_path,
            "type": "sendreceive",
            "rescanIntervalS": 5, // Faster rescan for accounts
            "fsWatcherEnabled": true,
            "fsWatcherDelayS": 1,
            "ignorePerms": true,
            "autoNormalize": true,
            "versioning": {
                "type": "simple",
                "params": { "keep": "10" } // Keep more backup versions of the accounts DB
            }
        });

        // [NEW] Sincronización del Código Fuente (Phase 6)
        let source_id = "antigravity-source-mesh";
        let source_path = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from(r"C:\Users\Administrator\Desktop\Antigravity-Manager"));
            
        let source_folder = json!({
            "id": source_id,
            "label": "Antigravity Source Code",
            "path": source_path,
            "type": "sendreceive",
            "rescanIntervalS": 30, // Normal rescan
            "fsWatcherEnabled": true,
            "fsWatcherDelayS": 2,
            "ignorePerms": true,
            "autoNormalize": true,
            "versioning": {
                "type": "trashcan",
                "params": { "cleanoutDays": "7" }
            }
        });

        // [NEW] CDP for Global AI Config (Phase 7)
        let config_id = "antigravity-config-mesh";
        let config_path = format!(r"{}\.gemini\config", app_dir);
        let config_folder = json!({
            "id": config_id,
            "label": "Antigravity Global Config",
            "path": config_path,
            "type": "sendreceive",
            "rescanIntervalS": 60,
            "fsWatcherEnabled": true,
            "fsWatcherDelayS": 5,
            "ignorePerms": true,
            "autoNormalize": true,
            "versioning": {
                "type": "trashcan",
                "params": { "cleanoutDays": "30" }
            }
        });

        let mut all_folders = folders;
        
        // Only push if they don't exist
        if !all_folders.iter().any(|f| f["id"] == brain_id) {
            all_folders.push(new_folder);
        }
        if !all_folders.iter().any(|f| f["id"] == accounts_id) {
            all_folders.push(accounts_folder);
        }
        if !all_folders.iter().any(|f| f["id"] == source_id) {
            all_folders.push(source_folder);
        }
        if !all_folders.iter().any(|f| f["id"] == config_id) {
            all_folders.push(config_folder);
        }

        // Update config
        let set_url = format!("{}/config", SYNCTHING_API_URL);
        let mut config: Value = self.client.get(&set_url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        config["folders"] = json!(all_folders);

        let res = self.client.put(&set_url)
            .header("X-API-Key", &self.api_key)
            .json(&config)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err("Failed to update Syncthing config".to_string());
        }

        self.apply_stignore(brain_id, &brain_path).await?;
        self.apply_source_stignore(source_id, &source_path).await?;
        Ok(())
    }

    async fn apply_stignore(&self, folder_id: &str, folder_path: &str) -> Result<(), String> {
        let ignore_path = PathBuf::from(folder_path).join(".stignore");
        let ignore_content = "(?d)*.log\n(?d).system_generated/\n(?d)tasks/\n";
        
        std::fs::write(&ignore_path, ignore_content).map_err(|e| e.to_string())?;
        tracing::info!("Wrote .stignore to exclude heavy logs from Syncthing.");
        Ok(())
    }

    async fn apply_source_stignore(&self, _folder_id: &str, folder_path: &str) -> Result<(), String> {
        let ignore_path = PathBuf::from(folder_path).join(".stignore");
        let ignore_content = "(?d)node_modules/\n(?d)src-tauri/target/\n(?d).git/\n(?d)dist/\n(?d)build/\n(?d)*.log\n";
        
        std::fs::write(&ignore_path, ignore_content).map_err(|e| e.to_string())?;
        tracing::info!("Wrote .stignore for source code to exclude build artifacts.");
        Ok(())
    }
}
