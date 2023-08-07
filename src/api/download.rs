use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{body::StreamBody, extract::Query, response::Response, Extension};
use http::{header, HeaderValue, StatusCode};
use serde::Deserialize;

use tokio::fs::File;
use tokio_util::io::ReaderStream;
use zip::CompressionMethod;

use crate::{config::Config, stream_utils::StreamWithClosure};

#[derive(Deserialize)]
pub struct Download {
    folder_name: String,
}

pub async fn download(
    config: Extension<Config>,
    download: Query<Download>,
) -> Result<Response<StreamBody<StreamWithClosure<ReaderStream<File>>>>, (StatusCode, String)> {
    let folder_name = download.folder_name.parse::<PathBuf>().map_err(|_| {
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

    let src_dir = config.root_folder_directory.join(&folder_name);

    if !src_dir.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("The folder does not exist"),
        ));
    }

    if !src_dir.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("That path is a file and not a folder"),
        ));
    }

    let dst_file = config.temp_zip_directory.join(format!(
        "{}-{}.zip",
        folder_name.to_string_lossy(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));

    std::fs::create_dir_all(&config.temp_zip_directory).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Failed to create temp zip directory `{:?}`: {:?}",
                config.temp_zip_directory, e
            ),
        )
    })?;

    let dst_file0 = dst_file.clone();

    if let Err(e) = tokio::task::spawn_blocking(move || {
        crate::zip_utils::zip_dir(&src_dir, &dst_file0, CompressionMethod::Stored)
    })
    .await
    .unwrap()
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to ZIP file: {:?}", e),
        ));
    }

    let file = match File::open(&dst_file).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);

    let dst_file0 = dst_file.clone();
    let stream = StreamWithClosure::new(
        stream,
        Box::new(move || {
            // stream has ended, delete the temp file
            std::fs::remove_file(&dst_file0).unwrap();
        }),
    );

    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let mut response = Response::new(body);

    for (header, value) in [
        (
            header::CONTENT_TYPE,
            format!("application/zip; charset=utf-8"),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"{}\"",
                dst_file.file_name().unwrap().to_str().unwrap()
            ),
        ),
    ] {
        response.headers_mut().insert(
            header,
            HeaderValue::try_from(value).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to convert string to header value: {}", e),
                )
            })?,
        );
    }

    Ok(response)
}
