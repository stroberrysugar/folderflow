use std::path::PathBuf;

use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::Serialize;

use crate::config::Config;

#[derive(Serialize)]
pub struct Response {
    created_at: DateTime<Utc>,
    name: String,
    full_path: PathBuf,
}

pub async fn list_folders(
    config: Extension<Config>,
) -> Result<Json<Vec<Response>>, (StatusCode, String)> {
    std::fs::create_dir_all(&config.root_folder_directory).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create directory: {:?}", e),
        )
    })?;

    let mut read_dir = tokio::fs::read_dir(&config.root_folder_directory)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read directory: {:?}", e),
            )
        })?;

    let mut responses = vec![];

    while let Ok(Some(n)) = read_dir.next_entry().await {
        let metadata = n.metadata().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read metadata of folder: {:?}", e),
            )
        })?;

        if !metadata.is_dir() {
            continue;
        }

        let created_at = metadata.created().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read metadata (created time) of folder: {:?}", e),
            )
        })?;

        responses.push(Response {
            name: n.file_name().to_string_lossy().to_string(),
            created_at: created_at.into(),
            full_path: n.path(),
        });
    }

    Ok(Json(responses))
}
