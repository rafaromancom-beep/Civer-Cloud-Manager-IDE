use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::path::Path;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

/// Devuelve el ejecutable principal
async fn download_release() -> Result<impl IntoResponse, StatusCode> {
    let file_path = Path::new("C:\\Program Files\\antigravity.civer.cloud\\civer_cloud_manager_ide_5_1.exe");
    
    // Si no existe ahí, buscar en target/release (para desarrollo)
    let path = if file_path.exists() {
        file_path.to_path_buf()
    } else {
        Path::new("target\\release\\civer_cloud_manager_ide_5_1.exe").to_path_buf()
    };

    if !path.exists() {
        tracing::error!("Release executable not found at {:?}", path);
        return Err(StatusCode::NOT_FOUND);
    }

    let file = match File::open(&path).await {
        Ok(file) => file,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"civer_cloud_manager_ide_5_1.exe\"",
        ),
    ];

    Ok((headers, body))
}

/// Genera un ZIP del código fuente en tiempo real y lo sirve
async fn download_source() -> Result<impl IntoResponse, StatusCode> {
    // Definimos el path temporal para el ZIP
    let zip_path = std::env::temp_dir().join("antigravity-manager-source.zip");
    
    // El root path es un nivel arriba de src-tauri
    let root_path = std::env::current_dir()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    tracing::info!("Generando archivo ZIP del código fuente en {:?}", zip_path);

    // Usamos tar de Windows (bsdtar) para comprimir
    let status = tokio::process::Command::new("tar")
        .current_dir(&root_path)
        .arg("-a")
        .arg("-c")
        .arg("-f")
        .arg(&zip_path)
        .arg("--exclude=node_modules")
        .arg("--exclude=src-tauri/target")
        .arg("--exclude=.git")
        .arg(".")
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            tracing::info!("ZIP generado exitosamente");
        }
        _ => {
            tracing::error!("Falló la generación del ZIP con tar");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let file = match File::open(&zip_path).await {
        Ok(file) => file,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/zip"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"antigravity_manager_source.zip\"",
        ),
    ];

    Ok((headers, body))
}

async fn download_github_msi() -> Result<impl IntoResponse, StatusCode> {
    // The URL is updated to point to the new white-labeled MSI filename
    let url = "https://github.com/PabloArboledai/draculabo-antigravity-manager-private-backup/releases/latest/download/Antigravity%20Civer%20Cloud%20IDE_4.4.7_x64_en-US.msi";
    
    let client = reqwest::Client::new();
    // Injected GitHub token so the download never fails even if the repo is private
    let res = client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, "token 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")
        .header(reqwest::header::USER_AGENT, "AntigravityManager")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch GitHub MSI: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !res.status().is_success() {
        tracing::error!("GitHub returned error status: {}", res.status());
        return Err(StatusCode::BAD_GATEWAY);
    }

    let stream = res.bytes_stream();
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"Antigravity Civer Cloud IDE_4.4.7_x64_en-US.msi\"",
        ),
    ];

    Ok((headers, body))
}

pub fn router() -> Router<crate::proxy::server::AppState> {
    Router::new()
        .route("/source", get(download_source))
        .route("/release", get(download_release))
        .route("/github_msi", get(download_github_msi))
        .route("/sync_mesh", axum::routing::post(sync_mesh))
}

async fn sync_mesh(body: String) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    use crate::models::account::MeshFullStateExport;
    let export: MeshFullStateExport = serde_json::from_str(&body).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    
    for account in export.accounts {
        let _ = crate::modules::account::save_account(&account);
    }
    
    if let Some(app_config) = export.app_config {
        let _ = crate::modules::config::save_app_config(&app_config);
    }
    
    // Trigger a refresh event globally
    crate::modules::log_bridge::emit_accounts_refreshed();
    
    Ok(axum::http::StatusCode::OK)
}
