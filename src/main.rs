/*

DONE 1. Folder uploading [should optionally accept folder name]
DONE 2. Folder downloading
DONE 3. Get list of folders along with the date/time created and FULL PATH
DONE 4. Get list of scripts
5. Execute script (select list of scripts) [can accept additional arguments]
   When entering the additional arguments, the full command executed should be displayed live
   (hence why we need the full path returned by the server-side)

*/

mod api;
mod config;
mod stream_utils;
mod zip_utils;

use std::net::SocketAddr;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Extension, Router,
};
use tower_http::{limit::RequestBodyLimitLayer, services::ServeDir};

#[tokio::main]
async fn main() {
    let config = self::config::get_config();

    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/download", get(self::api::download))
                .route("/upload", post(self::api::upload))
                .route("/execute", post(self::api::execute))
                .route("/list_folders", get(self::api::list_folders))
                .route("/list_scripts", get(self::api::list_scripts)),
        )
        .fallback_service(ServeDir::new("front-end/"))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(config.max_upload_size_in_bytes));

    axum::Server::bind(&config.listen_address)
        .serve(
            router
                .layer(Extension(config))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("Failed to start HTTP server");
}
