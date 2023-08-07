use std::path::PathBuf;

use axum::{Extension, Json};
use http::StatusCode;
use serde::Serialize;

use crate::config::Config;

#[derive(Serialize)]
pub struct Response {
    id: u16,
    friendly_name: String,
    path: PathBuf,
}

pub async fn list_scripts(
    config: Extension<Config>,
) -> Result<Json<Vec<Response>>, (StatusCode, String)> {
    Ok(Json(
        config
            .scripts
            .iter()
            .map(|x| Response {
                id: x.id,
                friendly_name: x.friendly_name.clone(),
                path: x.path_to_script.clone(),
            })
            .collect(),
    ))
}
