use std::{path::PathBuf, process::Command};

use axum::{extract::Query, Extension, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Deserialize)]
pub struct Execute {
    script_id: u16,
    folder_name: String,
}

#[derive(Serialize)]
pub struct Response {
    exit_status: String,
    stdout: String,
    stderr: String,
}

pub async fn execute(
    config: Extension<Config>,
    execute: Query<Execute>,
) -> Result<Json<Response>, (StatusCode, String)> {
    let script_config = config
        .scripts
        .iter()
        .find(|x| x.id == execute.script_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Script ID not found".to_string()))?;

    let folder_name = execute.folder_name.parse::<PathBuf>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("The folder name must be a valid directory name"),
        )
    })?;

    if folder_name.components().count() != 1 {
        return Err(
            (
                StatusCode::BAD_REQUEST,
                format!("The folder name must only have 1 component (i.e. it must just be a name and not contain any slashes)"),
            )
        );
    }

    let directory = config.root_folder_directory.join(folder_name);

    let mut command = Command::new(&script_config.path_to_script);

    command.arg(&directory).current_dir(&directory);

    let output = tokio::task::spawn_blocking(move || command.output())
        .await
        .unwrap()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute script: {:?}", e),
            )
        })?;

    Ok(Json(Response {
        exit_status: output.status.to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
    }))
}
