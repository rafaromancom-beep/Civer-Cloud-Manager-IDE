use crate::models::{Account, AppConfig, QuotaData};
use crate::modules;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

// 导出 proxy 命令
pub mod proxy;
// 导出 autostart 命令
pub mod autostart;
// 导出 cloudflared 命令
pub mod cloudflared;
// 导出 security 命令 (IP 监控)
pub mod security;
// 导出 proxy_pool 命令
pub mod proxy_pool;
// 导出 user_token 命令
pub mod user_token;
// 导出 patch 命令
pub mod patch;
pub use patch::*;

// 导出 telemetry 命令
pub mod telemetry;

use crate::modules::bootstrapper_config;

#[tauri::command]
pub async fn bootstrapper_get_config() -> Result<bootstrapper_config::BootstrapperConfig, String> {
    Ok(bootstrapper_config::load_config())
}

#[tauri::command]
pub async fn bootstrapper_save_config(config: bootstrapper_config::BootstrapperConfig) -> Result<(), String> {
    bootstrapper_config::save_config(&config)
}

#[tauri::command]
pub async fn bootstrapper_install_program(program_id: String) -> Result<(), String> {
    crate::modules::logger::log_info(&format!("Iniciando descarga e instalación real para: {}", program_id));
    let config = bootstrapper_config::load_config();
    let prog = config.programs.iter().find(|p| p.id == program_id)
        .ok_or_else(|| format!("Programa '{}' no encontrado en la configuración.", program_id))?;
        
    let url = prog.download_url.as_ref()
        .ok_or_else(|| format!("El programa '{}' no tiene URL de descarga configurada.", program_id))?;
        
    let target_path = prog.local_path.as_ref()
        .cloned()
        .unwrap_or_else(|| format!("C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables\\{}\\{}.exe", program_id, program_id));
        
    let target_dir = std::path::Path::new(&target_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "C:\\ProyectoCiverCloudUnificado\\Herramientas\\Apps-Portables".to_string());
        
    // Script PowerShell para descarga e instalacion portable dinamica y 100% silenciosa
    let ps_script = format!(
        "$ErrorActionPreference = 'Stop'; \
        New-Item -ItemType Directory -Force -Path '{}' | Out-Null; \
        $source = '{}'; \
        $target = '{}'; \
        $targetDir = '{}'; \
        if ($source -like 'http*') {{ \
          Write-Host ('Descargando desde ' + $source + '...'); \
          [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
          $tempFile = Join-Path $targetDir 'download_temp.file'; \
          Invoke-WebRequest -Uri $source -OutFile $tempFile -UserAgent 'Mozilla/5.0'; \
          if ($source -like '*.zip') {{ \
            Expand-Archive -Path $tempFile -DestinationPath $targetDir -Force; \
            Remove-Item $tempFile -Force; \
          }} elseif ($source -like '*.exe') {{ \
            Start-Process -FilePath $tempFile -ArgumentList ('/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /S /silent /quiet /DIR=\"' + $targetDir + '\"') -Wait -WindowStyle Hidden; \
            Remove-Item $tempFile -Force; \
          }} else {{ \
            Move-Item -Path $tempFile -Destination $target -Force; \
          }} \
        }} else {{ \
          Write-Host ('Instalando silenciosamente desde la bóveda oficial ' + $source + '...'); \
          if (Test-Path $source) {{ \
            if ($source -like '*.exe') {{ \
              Start-Process -FilePath $source -ArgumentList ('/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /S /silent /quiet /DIR=\"' + $targetDir + '\"') -Wait -WindowStyle Hidden; \
              if (-not (Test-Path $target)) {{ \
                $foundExe = Get-ChildItem -Path $targetDir -Filter '*.exe' -Recurse | Select-Object -First 1; \
                if ($foundExe) {{ Copy-Item -Path $foundExe.FullName -Destination $target -Force; }} \
              }} \
            }} elseif ($source -like '*.zip') {{ \
              Expand-Archive -Path $source -DestinationPath $targetDir -Force; \
            }} else {{ \
              Copy-Item -Path $source -Destination $target -Force; \
            }} \
          }} else {{ \
            throw ('El instalador oficial no existe en la bóveda: ' + $source); \
          }} \
        }}; \
        Write-Host 'Instalación silenciosa completada exitosamente.'",
        target_dir.replace("'", "''"),
        url.replace("'", "''"),
        target_path.replace("'", "''"),
        target_dir.replace("'", "''")
    );
    
    use std::process::Command;
    let mut cmd = Command::new("powershell");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    let output = cmd
        .args(&["-ExecutionPolicy", "Bypass", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Error al ejecutar PowerShell: {}", e))?;
        
    if output.status.success() {
        crate::modules::logger::log_info(&format!("Instalación de {} finalizada correctamente.", program_id));
        Ok(())
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let out_msg = String::from_utf8_lossy(&output.stdout);
        Err(format!("Fallo en la instalación: {}\n{}", err_msg, out_msg))
    }
}

#[tauri::command]
pub async fn bootstrapper_deploy_ag_cloner(program_id: String, target_node: String) -> Result<(), String> {
    crate::modules::logger::log_info(&format!("Starting ag-cloner deployment for: {} on node {}", program_id, target_node));
    
    // Spawn PowerShell to run ag-cloner.ps1 targeting the node
    use std::process::Command;
    let mut cmd = Command::new("powershell");
    cmd.args(&[
        "-ExecutionPolicy", "Bypass",
        "-File",
        r"C:\ProyectoCiverCloudUnificado\Herramientas\ag-cloner.ps1",
        "-TargetNode",
        &target_node
    ]);
    
    // Windows logic to prevent popups
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("Deployment to {} failed with exit code {}", target_node, status))
            }
        },
        Err(e) => Err(format!("Failed to execute ag-cloner: {}", e)),
    }
}

#[derive(serde::Serialize)]
pub struct VerifyResult {
    pub installed: bool,
    pub detected_path: Option<String>,
}

#[tauri::command]
pub async fn bootstrapper_verify_program(program_id: String, target_node: String) -> Result<VerifyResult, String> {
    let config = bootstrapper_config::load_config();
    
    // 1. Get the configured path
    let mut paths_to_check = Vec::new();
    if let Some(prog) = config.programs.iter().find(|p| p.id == program_id) {
        if let Some(ref path) = prog.local_path {
            paths_to_check.push(path.clone());
        }
    }

    // 2. Add smart fallback paths based on ID
    let vault_base = "C:\\ProyectoCiverCloudUnificado\\Herramientas";
    match program_id.as_str() {
        "antigravity_classic" => {
            // Antigravity 2.0 (v2.6.0 - Principal)
            paths_to_check.push("C:\\Users\\Administrator\\AppData\\Local\\Programs\\Antigravity\\Antigravity.exe".to_string());
            paths_to_check.push("C:\\Users\\Usuario\\AppData\\Local\\Programs\\Antigravity\\Antigravity.exe".to_string());
        }
        "antigravity_normal" => {
            // Antigravity IDE (v2.1.1 - Legacy)
            paths_to_check.push(format!("{}\\Apps-Portables\\AntigravityIDE-v2.1.1\\Antigravity.exe", vault_base));
        }
        "antigravity_cli" => {
            // Antigravity CLI - interfaz de linea de comandos (v1.1.11)
            paths_to_check.push(format!("{}\\Apps-Portables\\AntigravityIDE\\agy.exe", vault_base));
            paths_to_check.push("C:\\Users\\Administrator\\AppData\\Local\\Programs\\Antigravity\\resources\\agy.exe".to_string());
        }
        "antigravity_sdk" => {
            // Antigravity SDK - kit de desarrollo (v0.1.10)
            paths_to_check.push(format!("{}\\Apps-Portables\\Antigravity-Agent\\antigravity-agent.exe", vault_base));
        }
        "rclone" => {
            paths_to_check.push(format!("{}\\BajoNivel\\rclone.exe", vault_base));
            paths_to_check.push(format!("{}\\Apps-Portables\\Rclone\\rclone.exe", vault_base));
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Respaldos-y-Sync\\Omni-Backup-System\\bin\\rclone.exe".to_string());
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Sistema-Supervivencia-Backups\\3-Omni-Backup-System\\bin\\rclone.exe".to_string());
            paths_to_check.push("C:\\rclone\\rclone.exe".to_string());
        }
        "kopia" => {
            paths_to_check.push(format!("{}\\Apps-Portables\\KopiaConfig\\kopia.exe", vault_base));
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Respaldos-y-Sync\\Omni-Backup-System\\bin\\kopia.exe".to_string());
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Sistema-Supervivencia-Backups\\3-Omni-Backup-System\\bin\\kopia.exe".to_string());
            paths_to_check.push("C:\\Program Files\\Kopia\\KopiaUI.exe".to_string());
        }
        "autogravity" => {
            // AutoGravity - automatizacion IA satelital. Es proyecto Node/Electron compilado
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Desktop-y-Extensiones\\AutoGravity\\dist\\win-unpacked\\AutoGravity.exe".to_string());
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Desktop-y-Extensiones\\AutoGravity\\dist\\win-unpacked\\autogravity.exe".to_string());
            paths_to_check.push("C:\\Users\\Administrator\\AppData\\Local\\Programs\\autogravity\\AutoGravity.exe".to_string());
            // Si no está compilado, detectar el script de arranque
            paths_to_check.push("C:\\ProyectoCiverCloudUnificado\\Desktop-y-Extensiones\\AutoGravity\\package.json".to_string());
        }
        "tailscale" => {
            paths_to_check.push(format!("{}\\Apps-Portables\\Tailscale\\tailscale.exe", vault_base));
            paths_to_check.push("C:\\Program Files\\Tailscale\\tailscale.exe".to_string());
        }
        "protonvpn" => {
            paths_to_check.push(format!("{}\\Apps-Portables\\ProtonVPN\\Binaries\\ProtonVPN.Launcher.exe", vault_base));
            paths_to_check.push("C:\\Program Files\\Proton\\VPN\\v3.2.10\\ProtonVPN.Launcher.exe".to_string());
        }
        _ => {}
    }

    // 3. Verify intelligently
    if target_node == "localhost" {
        for path_str in paths_to_check {
            let p = std::path::Path::new(&path_str);
            if p.exists() {
                crate::modules::logger::log_info(&format!("Verified {} at {}", program_id, path_str));
                return Ok(VerifyResult { installed: true, detected_path: Some(path_str) });
            }
        }
    } else {
        // Verificacion remota via SSH (protocolo del ecosistema Civer Cloud)
        use std::process::Command;
        
        // Construir script PowerShell que verifica las rutas remotamente via SSH
        let paths_joined = paths_to_check.iter()
            .map(|p| format!("'{}'", p.replace('\\', "\\\\").replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",");
            
        let remote_script = format!(
            "foreach ($p in @({})) {{ if (Test-Path $p) {{ Write-Output $p; exit 0 }} }}; Write-Output 'NOT_FOUND'",
            paths_joined
        );
        
        let mut cmd = Command::new("ssh");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        
        let output = cmd
            .args(&[
                "-o", "ConnectTimeout=5",
                "-o", "StrictHostKeyChecking=no",
                "-o", "BatchMode=yes",
                &target_node,
                "powershell",
                "-NonInteractive",
                "-Command",
                &remote_script,
            ])
            .output();
            
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !stdout.is_empty() && stdout != "NOT_FOUND" {
                return Ok(VerifyResult { installed: true, detected_path: Some(stdout) });
            }
        }
    }
    
    Ok(VerifyResult { installed: false, detected_path: None })
}

#[tauri::command]
pub async fn bootstrapper_open_program(program_id: String, target_node: String, detected_path: String) -> Result<(), String> {
    use std::process::Command;

    if target_node == "localhost" {
        let path_obj = std::path::Path::new(&detected_path);
        if !path_obj.exists() {
            return Err(format!("El archivo ejecutable no existe en la ruta '{}'. Verifica o vuelve a instalar el programa.", detected_path));
        }

        let work_dir = path_obj.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

        let mut cmd = Command::new("cmd");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW para la consola cmd intermedia
        }
        
        match cmd
            .args(&[
                "/c",
                "start",
                "",
                "/d",
                &work_dir,
                &detected_path,
            ])
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error al abrir: {}", e)),
        }
    } else {
        // Apertura remota via SSH
        let remote_cmd = format!(
            "cmd /c start \"\" \"{}\"",
            detected_path.replace("\"", "\\\"")
        );
        let mut cmd = Command::new("ssh");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        match cmd
            .args(&[
                "-o", "ConnectTimeout=5",
                "-o", "StrictHostKeyChecking=no",
                "-o", "BatchMode=yes",
                &target_node,
                &remote_cmd,
            ])
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error al abrir en remoto: {}", e)),
        }
    }
}

#[tauri::command]
pub async fn bootstrapper_close_program(program_id: String, target_node: String, detected_path: String) -> Result<(), String> {
    use std::process::Command;
    let exe_name = std::path::Path::new(&detected_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
        
    let script = format!("Stop-Process -Name '{}' -Force -ErrorAction SilentlyContinue", exe_name);
    
    let mut cmd = Command::new("powershell");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    if target_node == "localhost" {
        match cmd.args(&["-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-Command", &script]).output() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to close program: {}", e)),
        }
    } else {
        match cmd
            .args(&[
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                &format!("Invoke-Command -ComputerName {} -ScriptBlock {{ {} }}", target_node, script)
            ]).output() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to close remote program: {}", e)),
        }
    }
}

#[tauri::command]
pub async fn bootstrapper_uninstall_program(program_id: String, target_node: String, detected_path: String) -> Result<(), String> {
    use std::process::Command;
    let exe_name = std::path::Path::new(&detected_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
        
    let script = format!(
        "Stop-Process -Name '{}' -Force -ErrorAction SilentlyContinue; \
        if (Test-Path '{}') {{ Remove-Item -Path '{}' -Recurse -Force -ErrorAction SilentlyContinue }}; \
        Write-Host 'Desinstalación completada.'",
        exe_name,
        detected_path.replace("'", "''"),
        detected_path.replace("'", "''")
    );
    
    let mut cmd = Command::new("powershell");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    if target_node == "localhost" {
        match cmd.args(&["-ExecutionPolicy", "Bypass", "-Command", &script]).output() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error en desinstalación: {}", e)),
        }
    } else {
        let remote_cmd = format!("powershell -NonInteractive -Command \"{}\"", script.replace("\"", "\\\""));
        let mut ssh_cmd = Command::new("ssh");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            ssh_cmd.creation_flags(0x08000000);
        }
        match ssh_cmd
            .args(&[
                "-o", "ConnectTimeout=5",
                "-o", "StrictHostKeyChecking=no",
                "-o", "BatchMode=yes",
                &target_node,
                &remote_cmd,
            ])
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error en desinstalación remota: {}", e)),
        }
    }
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    crate::modules::logger::log_info("Restarting app via shortcut...");
    app.restart();
}

#[tauri::command]
pub async fn exec_command(command: String, args: Vec<String>, cwd: Option<String>) -> Result<String, String> {
    use std::process::Command;
    let mut cmd = Command::new(&command);
    cmd.args(&args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    
    // Windows logic to prevent popups
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(stdout)
            } else {
                Err(format!("Exit {}: {}\n{}", output.status, stderr, stdout))
            }
        },
        Err(e) => Err(format!("Command execution failed: {}", e)),
    }
}

/// 列出所有账号
#[tauri::command]
pub async fn list_accounts(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Vec<Account>, String> {
    let mut accounts = tokio::task::spawn_blocking(move || modules::list_accounts())
        .await
        .unwrap_or_else(|_| Err("Task panicked".to_string()))?;

    // [FIX] Blend in-memory TokenManager rate limit status into the UI quota display
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        for account in &mut accounts {
            if let Some(reset_secs) = instance
                .token_manager
                .get_rate_limit_reset_seconds(&account.id)
            {
                if reset_secs > 0 {
                    if let Some(ref mut quota_data) = account.quota {
                        for model in &mut quota_data.models {
                            model.percentage = 0;
                            model.reset_time =
                                (chrono::Utc::now().timestamp() + reset_secs as i64).to_string();
                        }
                        // Optionally, add a UI flag if we want it to look completely blocked
                        // quota_data.is_forbidden = true;
                        // quota_data.forbidden_reason = Some(format!("Quota exhausted or rate limited (resets in {}s)", reset_secs));
                    }
                }
            }
        }
    }

    Ok(accounts)
}

/// 添加账号
#[tauri::command]
pub async fn add_account(
    app: tauri::AppHandle,
    _email: String,
    refresh_token: String,
) -> Result<Account, String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app.clone()),
    );

    let mut account = service.add_account(&refresh_token).await?;

    // 自动刷新配额
    let _ = internal_refresh_account_quota(&app, &mut account).await;

    // 重载账号池
    let _ = crate::commands::proxy::reload_proxy_accounts(
        app.state::<crate::commands::proxy::ProxyServiceState>(),
    )
    .await;

    Ok(account)
}

/// 删除账号
/// 删除账号
#[tauri::command]
pub async fn delete_account(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
) -> Result<(), String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app.clone()),
    );
    service.delete_account(&account_id)?;

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(())
}

/// 批量删除账号
#[tauri::command]
pub async fn delete_accounts(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_ids: Vec<String>,
) -> Result<(), String> {
    modules::logger::log_info(&format!(
        "收到批量删除请求，共 {} 个账号",
        account_ids.len()
    ));
    modules::account::delete_accounts(&account_ids).map_err(|e| {
        modules::logger::log_error(&format!("批量删除失败: {}", e));
        e
    })?;

    // 强制同步托盘
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(())
}

/// 重新排序账号列表
/// 根据传入的账号ID数组顺序更新账号排列
#[tauri::command]
pub async fn reorder_accounts(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_ids: Vec<String>,
) -> Result<(), String> {
    modules::logger::log_info(&format!(
        "收到账号重排序请求，共 {} 个账号",
        account_ids.len()
    ));
    modules::account::reorder_accounts(&account_ids).map_err(|e| {
        modules::logger::log_error(&format!("账号重排序失败: {}", e));
        e
    })?;

    // Reload pool to reflect new order if running
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;
    Ok(())
}

/// 切换账号
#[tauri::command]
pub async fn switch_account(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
    target_ide: Option<String>,
) -> Result<(), String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app.clone()),
    );

    service
        .switch_account(&account_id, target_ide.as_deref())
        .await?;

    // 同步托盘
    crate::modules::tray::update_tray_menus(&app);

    // [FIX #820] Notify proxy to clear stale session bindings and reload accounts
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(())
}

/// 获取当前账号
#[tauri::command]
pub async fn get_current_account() -> Result<Option<Account>, String> {
    // println!("🚀 Backend Command: get_current_account called"); // Commented out to reduce noise for frequent calls, relies on frontend log for frequency
    // Actually user WANTS to see it.
    modules::logger::log_info("Backend Command: get_current_account called");

    let account_id = modules::get_current_account_id()?;

    if let Some(id) = account_id {
        // modules::logger::log_info(&format!("   Found current account ID: {}", id));
        modules::load_account(&id).map(Some)
    } else {
        modules::logger::log_info("   No current account set");
        Ok(None)
    }
}

/// 导出账号（包含 refresh_token）
use crate::models::AccountExportResponse;

#[tauri::command]
pub async fn export_accounts(account_ids: Vec<String>) -> Result<AccountExportResponse, String> {
    tokio::task::spawn_blocking(move || modules::account::export_accounts_by_ids(&account_ids))
        .await
        .unwrap_or_else(|_| Err("Task panicked".to_string()))
}

#[tauri::command]
pub async fn export_full_state() -> Result<String, String> {
    use crate::models::account::MeshFullStateExport;
    let accounts = modules::account::list_accounts().unwrap_or_default();
    
    // Configs are optional for now, but we can load them if needed.
    let app_config = crate::modules::config::load_app_config().ok();
    
    let export = MeshFullStateExport {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        accounts,
        app_config,
    };

    serde_json::to_string(&export).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn import_full_state(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    payload: String
) -> Result<usize, String> {
    use crate::models::account::MeshFullStateExport;
    let export: MeshFullStateExport = serde_json::from_str(&payload)
        .map_err(|e| format!("Deserialization error: {}", e))?;
        
    let mut imported_count = 0;
    for mut account in export.accounts {
        match modules::account::save_account(&account) {
            Ok(_) => {
                imported_count += 1;
                // Add to index
                if let Ok(mut index) = modules::account::load_account_index() {
                    let summary = crate::models::AccountSummary {
                        id: account.id.clone(),
                        email: account.email.clone(),
                        name: account.name.clone(),
                        disabled: account.disabled,
                        proxy_disabled: account.proxy_disabled,
                        protected_models: account.protected_models.clone(),
                        created_at: account.created_at,
                        last_used: account.last_used,
                    };
                    if let Some(existing) = index.accounts.iter_mut().find(|a| a.id == account.id) {
                        *existing = summary;
                    } else {
                        index.accounts.push(summary);
                    }
                    let _ = modules::account::save_account_index(&index);
                }
            },
            Err(e) => crate::modules::logger::log_warn(&format!("Failed to import account {}: {}", account.email, e)),
        }
    }
    
    // Save App Config if present
    if let Some(app_config) = export.app_config {
        let _ = crate::modules::config::save_app_config(&app_config);
    }

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;
    
    // Notify frontend to refresh
    crate::modules::log_bridge::emit_accounts_refreshed();
    
    Ok(imported_count)
}

/// 内部辅助功能：在添加或导入账号后自动刷新一次额度
async fn internal_refresh_account_quota(
    app: &tauri::AppHandle,
    account: &mut Account,
) -> Result<QuotaData, String> {
    modules::logger::log_info(&format!("自动触发刷新配额: {}", account.email));

    // 使用带重试的查询 (Shared logic)
    match modules::account::fetch_quota_with_retry(account).await {
        Ok(quota) => {
            // 更新账号配额
            let _ = modules::update_account_quota(&account.id, quota.clone());
            // 更新托盘菜单
            crate::modules::tray::update_tray_menus(app);
            Ok(quota)
        }
        Err(e) => {
            modules::logger::log_warn(&format!("自动刷新配额失败 ({}): {}", account.email, e));
            Err(e.to_string())
        }
    }
}

/// 查询账号配额
#[tauri::command]
pub async fn fetch_account_quota(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
) -> crate::error::AppResult<QuotaData> {
    modules::logger::log_info(&format!("手动刷新配额请求: {}", account_id));
    let mut account =
        modules::load_account(&account_id).map_err(crate::error::AppError::Account)?;

    // 使用带重试的查询 (Shared logic)
    let mut quota = modules::account::fetch_quota_with_retry(&mut account).await?;

    // 4. 更新账号配额
    modules::update_account_quota(&account_id, quota.clone())
        .map_err(crate::error::AppError::Account)?;

    crate::modules::tray::update_tray_menus(&app);

    // 5. 同步到运行中的反代服务（如果已启动）
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        let _ = instance.token_manager.reload_account(&account_id).await;

        // [FIX] Blend TokenManager lockout state
        if let Some(reset_secs) = instance
            .token_manager
            .get_rate_limit_reset_seconds(&account_id)
        {
            if reset_secs > 0 {
                for model in &mut quota.models {
                    model.percentage = 0;
                    model.reset_time =
                        (chrono::Utc::now().timestamp() + reset_secs as i64).to_string();
                }
            }
        }
    }

    Ok(quota)
}

pub use modules::account::RefreshStats;

/// 刷新所有账号配额 (内部实现)
pub async fn refresh_all_quotas_internal(
    proxy_state: &crate::commands::proxy::ProxyServiceState,
    app_handle: Option<tauri::AppHandle>,
) -> Result<RefreshStats, String> {
    let stats = modules::account::refresh_all_quotas_logic().await?;

    // 同步到运行中的反代服务（如果已启动）
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        let _ = instance.token_manager.reload_all_accounts().await;
    }

    // 发送全局刷新事件给 UI (如果需要)
    if let Some(handle) = app_handle {
        use tauri::Emitter;
        let _ = handle.emit("accounts://refreshed", ());
    }

    Ok(stats)
}

/// 刷新所有账号配额 (Tauri Command)
#[tauri::command]
pub async fn refresh_all_quotas(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    app_handle: tauri::AppHandle,
) -> Result<RefreshStats, String> {
    refresh_all_quotas_internal(&proxy_state, Some(app_handle)).await
}
/// 获取设备指纹（当前 storage.json + 账号绑定）
#[tauri::command]
pub async fn get_device_profiles(
    account_id: String,
) -> Result<modules::account::DeviceProfiles, String> {
    modules::get_device_profiles(&account_id)
}

/// 绑定设备指纹（capture: 采集当前；generate: 生成新指纹），并写入 storage.json
#[tauri::command]
pub async fn bind_device_profile(
    account_id: String,
    mode: String,
) -> Result<crate::models::DeviceProfile, String> {
    modules::bind_device_profile(&account_id, &mode)
}

/// 预览生成一个指纹（不落盘）
#[tauri::command]
pub async fn preview_generate_profile() -> Result<crate::models::DeviceProfile, String> {
    Ok(crate::modules::device::generate_profile())
}

/// 使用给定指纹直接绑定
#[tauri::command]
pub async fn bind_device_profile_with_profile(
    account_id: String,
    profile: crate::models::DeviceProfile,
) -> Result<crate::models::DeviceProfile, String> {
    modules::bind_device_profile_with_profile(&account_id, profile, Some("generated".to_string()))
}

/// 将账号已绑定的指纹应用到 storage.json
#[tauri::command]
pub async fn apply_device_profile(
    account_id: String,
) -> Result<crate::models::DeviceProfile, String> {
    modules::apply_device_profile(&account_id)
}

/// 恢复最早的 storage.json 备份（近似“原始”状态）
#[tauri::command]
pub async fn restore_original_device() -> Result<String, String> {
    modules::restore_original_device()
}

/// 列出指纹版本
#[tauri::command]
pub async fn list_device_versions(
    account_id: String,
) -> Result<modules::account::DeviceProfiles, String> {
    modules::list_device_versions(&account_id)
}

/// 按版本恢复指纹
#[tauri::command]
pub async fn restore_device_version(
    account_id: String,
    version_id: String,
) -> Result<crate::models::DeviceProfile, String> {
    modules::restore_device_version(&account_id, &version_id)
}

/// 删除历史指纹（baseline 不可删）
#[tauri::command]
pub async fn delete_device_version(account_id: String, version_id: String) -> Result<(), String> {
    modules::delete_device_version(&account_id, &version_id)
}

/// 打开设备存储目录
#[tauri::command]
pub async fn open_device_folder(app: tauri::AppHandle) -> Result<(), String> {
    let dir = modules::device::get_storage_dir()?;
    let dir_str = dir
        .to_str()
        .ok_or("无法解析存储目录路径为字符串")?
        .to_string();
    app.opener()
        .open_path(dir_str, None::<&str>)
        .map_err(|e| format!("打开目录失败: {}", e))
}

/// 加载配置
#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    modules::load_app_config()
}

/// 保存配置
#[tauri::command]
pub async fn save_config(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    config: AppConfig,
) -> Result<(), String> {
    modules::save_app_config(&config)?;

    // 通知托盘配置已更新
    let _ = app.emit("config://updated", ());

    // 热更新正在运行的服务
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        // 更新模型映射
        instance.axum_server.update_mapping(&config.proxy).await;
        // 更新上游代理
        instance
            .axum_server
            .update_proxy(config.proxy.upstream_proxy.clone())
            .await;
        // 更新安全策略 (auth)
        instance.axum_server.update_security(&config.proxy).await;
        // 更新 z.ai 配置
        instance.axum_server.update_zai(&config.proxy).await;
        // 更新实验性配置
        instance
            .axum_server
            .update_experimental(&config.proxy)
            .await;
        // 更新调试日志配置
        instance
            .axum_server
            .update_debug_logging(&config.proxy)
            .await;
        // [NEW] 更新 User-Agent 配置
        instance.axum_server.update_user_agent(&config.proxy).await;
        // 更新 Thinking Budget 配置
        crate::proxy::update_thinking_budget_config(config.proxy.thinking_budget.clone());
        // [NEW] 更新全局系统提示词配置
        crate::proxy::update_global_system_prompt_config(config.proxy.global_system_prompt.clone());
        // [NEW] 更新全局图像思维模式配置
        crate::proxy::update_image_thinking_mode(config.proxy.image_thinking_mode.clone());
        // [NEW] 更新全局压缩等级配置
        crate::proxy::config::update_global_compression_level(
            config.proxy.experimental.compression_level.clone(),
            config.proxy.experimental.enable_usage_scaling,
        );
        crate::proxy::config::update_global_thresholds(
            config.proxy.experimental.context_compression_threshold_l1,
            config.proxy.experimental.context_compression_threshold_l2,
            config.proxy.experimental.context_compression_threshold_l3,
        );
        // 更新代理池配置
        instance
            .axum_server
            .update_proxy_pool(config.proxy.proxy_pool.clone())
            .await;
        // 更新熔断配置
        instance
            .token_manager
            .update_circuit_breaker_config(config.circuit_breaker.clone())
            .await;
        tracing::debug!("已同步热更新反代服务配置");
    }

    Ok(())
}

// --- OAuth 命令 ---

#[tauri::command]
pub async fn start_oauth_login(
    app_handle: tauri::AppHandle,
    oauth_client_key: Option<String>,
) -> Result<Account, String> {
    modules::logger::log_info("开始 OAuth 授权流程...");
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app_handle.clone()),
    );

    let mut account = service.start_oauth_login(oauth_client_key).await?;

    // 自动触发刷新额度
    let _ = internal_refresh_account_quota(&app_handle, &mut account).await;

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(
        app_handle.state::<crate::commands::proxy::ProxyServiceState>(),
    )
    .await;

    Ok(account)
}

/// 完成 OAuth 授权（不自动打开浏览器）
#[tauri::command]
pub async fn complete_oauth_login(app_handle: tauri::AppHandle) -> Result<Account, String> {
    modules::logger::log_info("完成 OAuth 授权流程 (manual)...");
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app_handle.clone()),
    );

    let mut account = service.complete_oauth_login().await?;

    // 自动触发刷新额度
    let _ = internal_refresh_account_quota(&app_handle, &mut account).await;

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(
        app_handle.state::<crate::commands::proxy::ProxyServiceState>(),
    )
    .await;

    Ok(account)
}

/// 预生成 OAuth 授权链接 (不打开浏览器)
#[tauri::command]
pub async fn prepare_oauth_url(
    app_handle: tauri::AppHandle,
    oauth_client_key: Option<String>,
) -> Result<String, String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app_handle.clone()),
    );
    service.prepare_oauth_url(oauth_client_key).await
}

#[tauri::command]
pub async fn cancel_oauth_login() -> Result<(), String> {
    modules::oauth_server::cancel_oauth_flow();
    Ok(())
}

/// 手动提交 OAuth Code (用于 Docker/远程环境无法自动回调时)
#[tauri::command]
pub async fn submit_oauth_code(code: String, state: Option<String>) -> Result<(), String> {
    modules::logger::log_info("收到手动提交 OAuth Code 请求");
    modules::oauth_server::submit_oauth_code(code, state).await
}

#[tauri::command]
pub async fn list_oauth_clients(
) -> Result<Vec<crate::modules::oauth::OAuthClientDescriptor>, String> {
    crate::modules::oauth::list_oauth_clients()
}

#[tauri::command]
pub async fn get_active_oauth_client() -> Result<String, String> {
    crate::modules::oauth::get_active_oauth_client_key()
}

#[tauri::command]
pub async fn set_active_oauth_client(client_key: String) -> Result<(), String> {
    crate::modules::oauth::set_active_oauth_client_key(&client_key)
}

// --- 导入命令 ---

#[tauri::command]
pub async fn import_v1_accounts(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Vec<Account>, String> {
    let accounts = modules::migration::import_from_v1().await?;

    // 对导入的账号尝试刷新一波
    for mut account in accounts.clone() {
        let _ = internal_refresh_account_quota(&app, &mut account).await;
    }

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(accounts)
}

#[tauri::command]
pub async fn import_from_db(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Account, String> {
    // 同步函数包装为 async
    let mut account = modules::migration::import_from_db(None).await?;

    // 既然是从数据库导入（即 IDE 当前账号），自动将其设为 Manager 的当前账号
    let account_id = account.id.clone();
    modules::account::set_current_account_id(&account_id)?;

    // 自动触发刷新额度
    let _ = internal_refresh_account_quota(&app, &mut account).await;

    // 刷新托盘图标展示
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(account)
}

#[tauri::command]
#[allow(dead_code)]
pub async fn import_custom_db(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    path: String,
) -> Result<Account, String> {
    // 调用重构后的自定义导入函数
    let mut account = modules::migration::import_from_custom_db_path(path).await?;

    // 自动设为当前账号
    let account_id = account.id.clone();
    modules::account::set_current_account_id(&account_id)?;

    // 自动触发刷新额度
    let _ = internal_refresh_account_quota(&app, &mut account).await;

    // 刷新托盘图标展示
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(account)
}

#[tauri::command]
pub async fn sync_account_from_db(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Option<Account>, String> {
    // Check if the current target is one we should not sync (like agy CLI)
    let index = modules::account::load_account_index()?;
    let current_target = index.current_target_ide.as_deref();
    if current_target == Some("agy") {
        modules::logger::log_info("Auto-sync skipped: current target is agy CLI");
        return Ok(None);
    }

    // 1. 获取 DB 中的 Refresh Token
    let db_refresh_token = match modules::migration::get_refresh_token_from_db(current_target) {
        Ok(token) => token,
        Err(e) => {
            modules::logger::log_info(&format!("自动同步跳过: {}", e));
            return Ok(None);
        }
    };

    // 2. 获取 Manager 当前账号
    let curr_account = modules::account::get_current_account()?;

    // 3. 对比：如果 Refresh Token 相同，说明账号没变，无需导入
    if let Some(acc) = curr_account {
        if acc.token.refresh_token == db_refresh_token {
            // 账号未变，由于已经是周期性任务，我们可以选择性刷新一下配额，或者直接返回
            // 这里为了节省 API 流量，直接返回
            return Ok(None);
        }
        modules::logger::log_info(&format!(
            "检测到账号切换 ({} -> DB新账号)，正在同步...",
            acc.email
        ));
    } else {
        modules::logger::log_info("检测到新登录账号，正在自动同步...");
    }

    // 4. 执行完整导入
    let mut account = modules::migration::import_from_db(current_target).await?;

    // 既然是从数据库导入，自动将其设为 Manager 的当前账号并保留当前 target
    let account_id = account.id.clone();
    modules::account::set_current_account_id_with_target(&account_id, current_target)?;

    // 自动触发刷新额度
    let _ = internal_refresh_account_quota(&app, &mut account).await;

    // 刷新托盘图标展示
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    let _ = crate::commands::proxy::reload_proxy_accounts(proxy_state).await;

    Ok(Some(account))
}

fn resolve_existing_or_parent(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|e| format!("failed_to_resolve_path: {}", e));
    }

    let parent = path
        .parent()
        .ok_or_else(|| "invalid_path: missing parent directory".to_string())?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| format!("failed_to_resolve_parent: {}", e))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "invalid_path: missing file name".to_string())?;
    Ok(canonical_parent.join(file_name))
}

fn is_sensitive_path(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    let sensitive_prefixes = [
        "/etc/",
        "/var/spool/cron",
        "/root/",
        "/proc/",
        "/sys/",
        "/dev/",
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\users\\administrator",
        "c:\\pagefile.sys",
    ];

    sensitive_prefixes
        .iter()
        .any(|prefix| lower == *prefix || lower.starts_with(prefix))
}

fn validate_user_json_path(path: &str, must_exist: bool) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    if requested.as_os_str().is_empty() {
        return Err("invalid_path: empty path".to_string());
    }
    if !requested.is_absolute() {
        return Err("invalid_path: absolute path is required".to_string());
    }

    let resolved = resolve_existing_or_parent(&requested)?;
    if is_sensitive_path(&resolved) {
        return Err("security_denied: sensitive system path is not allowed".to_string());
    }

    let is_json = resolved
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    if !is_json {
        return Err("invalid_path: only .json files are allowed".to_string());
    }

    if must_exist {
        let metadata = std::fs::metadata(&resolved)
            .map_err(|e| format!("failed_to_read_file_metadata: {}", e))?;
        if !metadata.is_file() {
            return Err("invalid_path: expected a regular file".to_string());
        }
    }

    Ok(resolved)
}

/// 保存文本文件 (绕过前端 Scope 限制)
#[tauri::command]
pub async fn save_text_file(path: String, content: String) -> Result<(), String> {
    let path = validate_user_json_path(&path, false)?;
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 读取文本文件 (绕过前端 Scope 限制)
#[tauri::command]
pub async fn read_text_file(path: String) -> Result<String, String> {
    let path = validate_user_json_path(&path, true)?;
    std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 清理日志缓存
#[tauri::command]
pub async fn clear_log_cache() -> Result<(), String> {
    modules::logger::clear_logs()
}

/// 清理 Antigravity 应用缓存
/// 用于解决登录失败、版本验证错误等问题
#[tauri::command]
pub async fn clear_antigravity_cache() -> Result<modules::cache::ClearResult, String> {
    modules::cache::clear_antigravity_cache(None)
}

/// 获取 Antigravity 缓存路径列表（用于预览）
#[tauri::command]
pub async fn get_antigravity_cache_paths() -> Result<Vec<String>, String> {
    Ok(modules::cache::get_existing_cache_paths()
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

/// 打开数据目录
#[tauri::command]
pub async fn open_data_folder() -> Result<(), String> {
    let path = modules::account::get_data_dir()?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        use crate::utils::command::CommandExtWrapper;
        std::process::Command::new("explorer")
            .creation_flags_windows()
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(())
}

/// 获取数据目录绝对路径
#[tauri::command]
pub async fn get_data_dir_path() -> Result<String, String> {
    let path = modules::account::get_data_dir()?;
    Ok(path.to_string_lossy().to_string())
}

/// 显示主窗口
#[tauri::command]
pub async fn show_main_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())
}

/// 设置窗口主题（用于同步 Windows 标题栏按钮颜色）
#[tauri::command]
pub async fn set_window_theme(window: tauri::Window, theme: String) -> Result<(), String> {
    use tauri::Theme;

    let tauri_theme = match theme.as_str() {
        "dark" => Some(Theme::Dark),
        "light" => Some(Theme::Light),
        _ => None, // system default
    };

    window.set_theme(tauri_theme).map_err(|e| e.to_string())
}

/// 获取 Antigravity 可执行文件路径
#[tauri::command]
pub async fn get_antigravity_path(bypass_config: Option<bool>) -> Result<String, String> {
    // 1. 优先从配置查询 (除非明确要求绕过)
    if bypass_config != Some(true) {
        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(path) = config.antigravity_executable {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    // 2. 执行实时探测
    match crate::modules::process::get_antigravity_executable_path(None) {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("未找到 Antigravity 安装路径".to_string()),
    }
}

/// 获取 Antigravity CLI (agy) 可执行文件路径
#[tauri::command]
pub async fn get_antigravity_cli_path(bypass_config: Option<bool>) -> Result<String, String> {
    // 1. 优先从配置查询 (除非明确要求绕过)
    if bypass_config != Some(true) {
        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(path) = config.antigravity_cli_executable {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    // 2. 执行实时探测
    match crate::modules::process::get_antigravity_cli_executable_path() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("未找到 Antigravity CLI (agy) 安装路径".to_string()),
    }
}

/// 获取 Antigravity 启动参数
#[tauri::command]
pub async fn get_antigravity_args() -> Result<Vec<String>, String> {
    match crate::modules::process::get_args_from_running_process(None) {
        Some(args) => Ok(args),
        None => Err("未找到正在运行的 Antigravity 进程".to_string()),
    }
}

/// 检测更新响应结构
pub use crate::modules::update_checker::UpdateInfo;

/// 检测 GitHub releases 更新
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    modules::logger::log_info("收到前端触发的更新检查请求");
    crate::modules::update_checker::check_for_updates().await
}

#[tauri::command]
pub async fn should_check_updates() -> Result<bool, String> {
    let settings = crate::modules::update_checker::load_update_settings()?;
    Ok(crate::modules::update_checker::should_check_for_updates(
        &settings,
    ))
}

#[tauri::command]
pub async fn update_last_check_time() -> Result<(), String> {
    crate::modules::update_checker::update_last_check_time()
}

/// 检测是否通过 Homebrew Cask 安装
#[tauri::command]
pub async fn check_homebrew_installation() -> Result<bool, String> {
    Ok(crate::modules::update_checker::is_homebrew_installed())
}

/// 检测是否以 AppImage 方式运行（Linux 专用）
/// Tauri 的原生更新器在 Linux 上只支持 AppImage，
/// RPM/DEB 安装的用户不应触发原生自动更新以避免 ENOEXEC 错误。
#[tauri::command]
pub async fn check_appimage_installation() -> Result<bool, String> {
    Ok(crate::modules::update_checker::is_appimage_running())
}

/// 通过 Homebrew Cask 升级应用
#[tauri::command]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    modules::logger::log_info("收到前端触发的 Homebrew 升级请求");
    crate::modules::update_checker::brew_upgrade_cask().await
}

/// 获取更新设置
#[tauri::command]
pub async fn get_update_settings() -> Result<crate::modules::update_checker::UpdateSettings, String>
{
    crate::modules::update_checker::load_update_settings()
}

/// 保存更新设置
#[tauri::command]
pub async fn save_update_settings(
    settings: crate::modules::update_checker::UpdateSettings,
) -> Result<(), String> {
    crate::modules::update_checker::save_update_settings(&settings)
}

/// 切换账号的反代禁用状态
#[tauri::command]
pub async fn toggle_proxy_status(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
    enable: bool,
    reason: Option<String>,
) -> Result<(), String> {
    modules::logger::log_info(&format!(
        "切换账号反代状态: {} -> {}",
        account_id,
        if enable { "启用" } else { "禁用" }
    ));

    // 1. 读取账号文件
    let data_dir = modules::account::get_data_dir()?;
    let account_path = data_dir
        .join("accounts")
        .join(format!("{}.json", account_id));

    if !account_path.exists() {
        return Err(format!("账号文件不存在: {}", account_id));
    }

    let content =
        std::fs::read_to_string(&account_path).map_err(|e| format!("读取账号文件失败: {}", e))?;

    let mut account_json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析账号文件失败: {}", e))?;

    // 2. 更新 proxy_disabled 字段
    if enable {
        // 启用反代
        account_json["proxy_disabled"] = serde_json::Value::Bool(false);
        account_json["proxy_disabled_reason"] = serde_json::Value::Null;
        account_json["proxy_disabled_at"] = serde_json::Value::Null;
    } else {
        // 禁用反代
        let now = chrono::Utc::now().timestamp();
        account_json["proxy_disabled"] = serde_json::Value::Bool(true);
        account_json["proxy_disabled_at"] = serde_json::Value::Number(now.into());
        account_json["proxy_disabled_reason"] =
            serde_json::Value::String(reason.unwrap_or_else(|| "用户手动禁用".to_string()));
    }

    // 3. 保存到磁盘
    let json_str = serde_json::to_string_pretty(&account_json)
        .map_err(|e| format!("序列化账号数据失败: {}", e))?;
    std::fs::write(&account_path, json_str).map_err(|e| format!("写入账号文件失败: {}", e))?;

    modules::logger::log_info(&format!(
        "账号反代状态已更新: {} ({})",
        account_id,
        if enable { "已启用" } else { "已禁用" }
    ));

    // 4. 如果反代服务正在运行,立刻同步到内存池（避免禁用后仍被选中）
    {
        let instance_lock = proxy_state.instance.read().await;
        if let Some(instance) = instance_lock.as_ref() {
            // 如果禁用的是当前固定账号，则自动关闭固定模式（内存 + 配置持久化）
            if !enable {
                let pref_id = instance.token_manager.get_preferred_account().await;
                if pref_id.as_deref() == Some(&account_id) {
                    instance.token_manager.set_preferred_account(None).await;

                    if let Ok(mut cfg) = crate::modules::config::load_app_config() {
                        if cfg.proxy.preferred_account_id.as_deref() == Some(&account_id) {
                            cfg.proxy.preferred_account_id = None;
                            let _ = crate::modules::config::save_app_config(&cfg);
                        }
                    }
                }
            }

            instance
                .token_manager
                .reload_account(&account_id)
                .await
                .map_err(|e| format!("同步账号失败: {}", e))?;
        }
    }

    // 5. 更新托盘菜单
    crate::modules::tray::update_tray_menus(&app);

    Ok(())
}

/// 预热所有可用账号
#[tauri::command]
pub async fn warm_up_all_accounts() -> Result<String, String> {
    modules::quota::warm_up_all_accounts().await
}

/// 预热指定账号
#[tauri::command]
pub async fn warm_up_account(account_id: String) -> Result<String, String> {
    modules::quota::warm_up_account(&account_id).await
}

/// 更新账号自定义标签
#[tauri::command]
pub async fn update_account_label(account_id: String, label: String) -> Result<(), String> {
    // 验证标签长度（按字符数计算，支持中文）
    if label.chars().count() > 15 {
        return Err("标签长度不能超过15个字符".to_string());
    }

    modules::logger::log_info(&format!(
        "更新账号标签: {} -> {:?}",
        account_id,
        if label.is_empty() { "无" } else { &label }
    ));

    // 1. 读取账号文件
    let data_dir = modules::account::get_data_dir()?;
    let account_path = data_dir
        .join("accounts")
        .join(format!("{}.json", account_id));

    if !account_path.exists() {
        return Err(format!("账号文件不存在: {}", account_id));
    }

    let content =
        std::fs::read_to_string(&account_path).map_err(|e| format!("读取账号文件失败: {}", e))?;

    let mut account_json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析账号文件失败: {}", e))?;

    // 2. 更新 custom_label 字段
    if label.is_empty() {
        account_json["custom_label"] = serde_json::Value::Null;
    } else {
        account_json["custom_label"] = serde_json::Value::String(label.clone());
    }

    // 3. 保存到磁盘
    let json_str = serde_json::to_string_pretty(&account_json)
        .map_err(|e| format!("序列化账号数据失败: {}", e))?;
    std::fs::write(&account_path, json_str).map_err(|e| format!("写入账号文件失败: {}", e))?;

    modules::logger::log_info(&format!(
        "账号标签已更新: {} ({})",
        account_id,
        if label.is_empty() {
            "已清除".to_string()
        } else {
            label
        }
    ));

    Ok(())
}

// ============================================================================
// HTTP API 设置命令
// ============================================================================

/// 获取 HTTP API 设置
#[tauri::command]
pub async fn get_http_api_settings() -> Result<crate::modules::http_api::HttpApiSettings, String> {
    crate::modules::http_api::load_settings()
}

/// 保存 HTTP API 设置
#[tauri::command]
pub async fn save_http_api_settings(
    settings: crate::modules::http_api::HttpApiSettings,
) -> Result<(), String> {
    crate::modules::http_api::save_settings(&settings)
}

// ============================================================================
// Token Statistics Commands
// ============================================================================

pub use crate::modules::token_stats::{AccountTokenStats, TokenStatsAggregated, TokenStatsSummary};

#[tauri::command]
pub async fn get_token_stats_hourly(hours: i64) -> Result<Vec<TokenStatsAggregated>, String> {
    crate::modules::token_stats::get_hourly_stats(hours)
}

#[tauri::command]
pub async fn get_token_stats_daily(days: i64) -> Result<Vec<TokenStatsAggregated>, String> {
    crate::modules::token_stats::get_daily_stats(days)
}

#[tauri::command]
pub async fn get_token_stats_weekly(weeks: i64) -> Result<Vec<TokenStatsAggregated>, String> {
    crate::modules::token_stats::get_weekly_stats(weeks)
}

#[tauri::command]
pub async fn get_token_stats_by_account(hours: i64) -> Result<Vec<AccountTokenStats>, String> {
    crate::modules::token_stats::get_account_stats(hours)
}

#[tauri::command]
pub async fn get_token_stats_summary(hours: i64) -> Result<TokenStatsSummary, String> {
    crate::modules::token_stats::get_summary_stats(hours)
}

#[tauri::command]
pub async fn get_token_stats_by_model(
    hours: i64,
) -> Result<Vec<crate::modules::token_stats::ModelTokenStats>, String> {
    crate::modules::token_stats::get_model_stats(hours)
}

#[tauri::command]
pub async fn get_token_stats_model_trend_hourly(
    hours: i64,
) -> Result<Vec<crate::modules::token_stats::ModelTrendPoint>, String> {
    crate::modules::token_stats::get_model_trend_hourly(hours)
}

#[tauri::command]
pub async fn get_token_stats_model_trend_daily(
    days: i64,
) -> Result<Vec<crate::modules::token_stats::ModelTrendPoint>, String> {
    crate::modules::token_stats::get_model_trend_daily(days)
}

#[tauri::command]
pub async fn get_token_stats_account_trend_hourly(
    hours: i64,
) -> Result<Vec<crate::modules::token_stats::AccountTrendPoint>, String> {
    crate::modules::token_stats::get_account_trend_hourly(hours)
}

#[tauri::command]
pub async fn get_token_stats_account_trend_daily(
    days: i64,
) -> Result<Vec<crate::modules::token_stats::AccountTrendPoint>, String> {
    crate::modules::token_stats::get_account_trend_daily(days)
}

#[tauri::command]
pub async fn query_transit_info(url: String, key: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;

    if status.is_success() {
        Ok(text)
    } else {
        Err(format!("HTTP {}: {}", status, text))
    }
}

#[tauri::command]
pub async fn uninstall_program(app_handle: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let script_path = std::env::temp_dir().join("uninstall_antigravity.ps1");
        let script = r#"
Stop-Process -Name "civer_cloud_manager_ide_5_1" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "antigravity.civer.cloud" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Remove-Item -Path "$env:LOCALAPPDATA\cloud.civer.antigravity" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "$env:APPDATA\cloud.civer.antigravity" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "$env:USERPROFILE\Desktop\Antigravity*.lnk" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Antigravity" -Force -Recurse -ErrorAction SilentlyContinue
$script_path = $MyInvocation.MyCommand.Path
Remove-Item -Path $script_path -Force -ErrorAction SilentlyContinue
"#;
        std::fs::write(&script_path, script).map_err(|e| e.to_string())?;

        let args = format!(
            "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File \"{}\"",
            script_path.display()
        );

        std::process::Command::new("powershell")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg(&format!("Start-Process powershell -ArgumentList '{}'", args))
            .spawn()
            .map_err(|e| format!("Failed to launch uninstaller: {}", e))?;
        
        app_handle.exit(0);
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Uninstall only supported on Windows".into())
    }
}

// ============================================================================
// Mesh Remote Execution Commands
// ============================================================================

#[tauri::command]
pub async fn mesh_execute_action(ip: String, action: String, payload: Option<String>) -> Result<String, String> {
    crate::modules::logger::log_info(&format!("mesh_execute_action to IP {} with action {}", ip, action));
    
    // Check if it's an interactive protocol connection (RDP/SSH)
    if action == "CONNECT_RDP" {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("mstsc")
                .arg(format!("/v:{}", ip))
                .spawn()
                .map_err(|e| format!("Error launching RDP: {}", e))?;
            return Ok("RDP Client launched".to_string());
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Err("RDP direct launch only supported from Windows client".to_string());
        }
    } else if action == "CONNECT_SSH" {
        #[cfg(target_os = "windows")]
        {
            let user = payload.unwrap_or_else(|| "root".to_string());
            
            // Embed the SSH key inside the executable so it's always available
            let key_content = include_str!("../../assets/id_rsa_antigravity");
            
            // Write to a temporary file with restricted permissions (Windows handles this mostly in temp)
            let temp_dir = std::env::temp_dir();
            let key_path = temp_dir.join("id_rsa_antigravity_mesh.tmp");
            std::fs::write(&key_path, key_content)
                .map_err(|e| format!("Failed to write temp SSH key: {}", e))?;
                
            std::process::Command::new("cmd")
                .arg("/c")
                .arg("start")
                .arg("ssh")
                .arg("-i")
                .arg(key_path.to_string_lossy().to_string())
                .arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg(format!("{}@{}", user, ip))
                .spawn()
                .map_err(|e| format!("Error launching SSH: {}", e))?;
            return Ok("SSH Client launched with Embedded Vault keys".to_string());
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Err("SSH direct launch only supported from Windows client currently".to_string());
        }
    }
    
    // Background / Service Actions
    let cmd_text = match action.as_str() {
        "START_APP" => {
            // This is just a simulated command for now, assuming the app is in standard locations
            if ip == "192.168.1.93" {
                "nohup /opt/antigravity/antigravity.civer.cloud > /dev/null 2>&1 &".to_string()
            } else {
                "Start-Process -FilePath \"C:\\Program Files\\antigravity.civer.cloud\\antigravity.civer.cloud.exe\"".to_string()
            }
        },
        "STOP_APP" => {
            if ip == "192.168.1.93" {
                "killall antigravity.civer.cloud".to_string()
            } else {
                "Stop-Process -Name \"antigravity.civer.cloud\" -Force".to_string()
            }
        },
        "INSTALL_APP" => {
            // Simulamos la instalación descargando el script de instalación automática
            if ip == "192.168.1.93" {
                "curl -sL https://antigravity.civer.cloud/install.sh | bash".to_string()
            } else {
                "Invoke-WebRequest -Uri https://antigravity.civer.cloud/install.ps1 -OutFile $env:TEMP\\install.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\\install.ps1".to_string()
            }
        },
        "UNINSTALL_APP" => {
            if ip == "192.168.1.93" {
                "rm -rf /opt/antigravity".to_string()
            } else {
                "Stop-Process -Name \"antigravity.civer.cloud\" -Force; Remove-Item -Path \"C:\\Program Files\\antigravity.civer.cloud\" -Recurse -Force".to_string()
            }
        },
        "RUN_PROCESS" => {
            payload.unwrap_or_default()
        },
        _ => return Err(format!("Unknown action: {}", action))
    };
    
    // We add this to the durable command queue in DB.
    // The reconnection loop in node_manager will pick it up and execute it when the node is online.
    let id = uuid::Uuid::new_v4().to_string();
    match crate::modules::command_runner_db::insert_command(&id, &ip, &cmd_text) {
        Ok(_) => Ok(format!("Command queued with ID: {}", id)),
        Err(e) => Err(format!("Failed to queue command: {}", e))
    }
}

// --- Mesh Radar Commands ---

#[tauri::command]
pub async fn get_mesh_nodes() -> Result<Vec<crate::modules::command_runner_db::MeshNode>, String> {
    crate::modules::command_runner_db::get_mesh_nodes()
}

#[tauri::command]
pub async fn add_mesh_node(ip: String, name: String, protocol: String) -> Result<(), String> {
    crate::modules::command_runner_db::add_or_update_mesh_node(&ip, &name, &protocol)
}

#[tauri::command]
pub async fn delete_mesh_node(ip: String) -> Result<(), String> {
    crate::modules::command_runner_db::delete_mesh_node(&ip)
}

#[tauri::command]
pub async fn update_mesh_node(ip: String, name: String, protocol: String) -> Result<(), String> {
    crate::modules::command_runner_db::add_or_update_mesh_node(&ip, &name, &protocol)
}

#[tauri::command]
pub async fn wipe_and_update_remote_node(ip: String) -> Result<String, String> {
    crate::modules::logger::log_info(&format!("Starting remote wipe and update for node: {}", ip));
    
    // Save the script to a temporary file
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
Write-Host 'Cerrando procesos...'
Get-Process | Where-Object { $_.Name -match 'antigravity|civer' } | Stop-Process -Force

Write-Host 'Purgando carpetas legacy...'
Remove-Item -Path "$env:LOCALAPPDATA\Programs\antigravity" -Recurse -Force
Remove-Item -Path "$env:LOCALAPPDATA\Antigravity Manager Civer Cloud" -Recurse -Force
Remove-Item -Path "$env:LOCALAPPDATA\Civer Cloud Manager" -Recurse -Force
Remove-Item -Path "C:\Program Files\Civer Cloud Manager" -Recurse -Force
Remove-Item -Path "C:\Program Files\Civer Cloud Manager IDE" -Recurse -Force

Write-Host 'Purgando accesos directos legacy...'
Get-ChildItem -Path "$env:USERPROFILE\Desktop" -Filter '*Antigravity*.lnk' | Remove-Item -Force
Get-ChildItem -Path "$env:PUBLIC\Desktop" -Filter '*Antigravity*.lnk' | Remove-Item -Force
Get-ChildItem -Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Filter '*Antigravity*.lnk' -Recurse | Remove-Item -Force
Remove-Item -Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Antigravity IDE" -Force -Recurse

$installer = 'C:\Users\Administrator\Civer_IDE_v5_setup.exe'
Invoke-WebRequest -Uri 'http://100.111.186.118:8123/Civer%20Cloud%20Manager%20IDE_5.0.0_x64-setup.exe' -OutFile $installer -UseBasicParsing

if (Test-Path $installer) {
    Start-Process -FilePath $installer -ArgumentList '/S' -Wait
    Start-Sleep -Seconds 3
    
    $exe = "$env:LOCALAPPDATA\Civer Cloud Manager IDE\civer_cloud_manager_ide.exe"
    if (-not (Test-Path $exe)) {
        $exe = "C:\Program Files\Civer Cloud Manager IDE\civer_cloud_manager_ide.exe"
    }
    
    if (Test-Path $exe) {
        Write-Host "Inyectando UI interactiva en la pantalla del usuario (Session 0 Bypass)..."
        $actionApp = New-ScheduledTaskAction -Execute $exe
        $principal = New-ScheduledTaskPrincipal -UserId "Usuario" -LogonType Interactive
        $task = New-ScheduledTask -Action $actionApp -Principal $principal
        
        Register-ScheduledTask -TaskName "PopAppV5" -InputObject $task -Force | Out-Null
        Start-ScheduledTask -TaskName "PopAppV5"
        
        Start-Sleep -Seconds 3
        Unregister-ScheduledTask -TaskName "PopAppV5" -Confirm:$false
    }
}
"#;

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("remote_wipe.ps1");
    if let Err(e) = std::fs::write(&script_path, script) {
        return Err(format!("Failed to write script: {}", e));
    }

    let ps_command = format!(
        r#" = ConvertTo-SecureString 'AgenteAccess!2026' -AsPlainText -Force;  = New-Object System.Management.Automation.PSCredential ('Usuario', ); Invoke-Command -ComputerName {} -Credential  -FilePath '{}'"#,
        ip,
        script_path.display()
    );

    let output = std::process::Command::new("powershell")
        .args(&["-Command", &ps_command])
        .output()
        .map_err(|e| format!("Failed to execute powershell: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    crate::modules::logger::log_info(&format!("Wipe output: {}", stdout));
    if !stderr.is_empty() {
        crate::modules::logger::log_warn(&format!("Wipe stderr: {}", stderr));
    }

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Exit {}: {}", output.status, stderr))
    }
}
