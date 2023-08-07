use std::{
    io::ErrorKind,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Multipart, Query},
    Extension,
};
use futures::TryStreamExt;
use http::StatusCode;
use serde::Deserialize;
use tokio::fs::OpenOptions;
use tokio_util::io::StreamReader;

use crate::config::Config;

#[derive(Deserialize)]
pub struct Upload {
    folder_name: Option<String>,
}

pub async fn upload(
    config: Extension<Config>,
    upload: Query<Upload>,
    mut multipart: Multipart,
) -> Result<(), (StatusCode, String)> {
    let folder_name = upload
        .folder_name
        .clone()
        .unwrap_or_else(|| {
            format!(
                "folder-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )
        })
        .parse::<PathBuf>()
        .map_err(|_| {
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

    let directory_to_put_files_in = config.root_folder_directory.join(folder_name);

    std::fs::create_dir_all(&directory_to_put_files_in).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create folder: {:?}", e),
        )
    })?;

    loop {
        let field = match multipart
            .next_field()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
        {
            Some(n) => n,
            None => break,
        };

        let name = field.name().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("The field name is invalid"),
            )
        })?;
        let name = name.parse::<PathBuf>().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                format!("The file name `{}` is invalid", name),
            )
        })?;

        if name.components().count() != 1 {
            return Err(
                    (
                        StatusCode::BAD_REQUEST,
                        format!("The file name must only have 1 component (i.e. it must just be a name and not contain any slashes)"),
                    )
                );
        }

        let final_download_path = directory_to_put_files_in.join(name);

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&final_download_path)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create file `{:?}`: {:?}", final_download_path, e),
                )
            })?;

        let mut field =
            StreamReader::new(field.map_err(|e| std::io::Error::new(ErrorKind::Other, e)));

        tokio::io::copy(&mut field, &mut file).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Failed to write data to file `{:?}`: {:?}",
                    final_download_path, e
                ),
            )
        })?;
    }

    Ok(())
}
