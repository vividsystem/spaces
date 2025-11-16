use std::path::Path;

use anyhow::Context;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::HeaderValue,
    routing::{delete, get},
};

mod errors;
mod files;
mod spaces;

use sqlx::PgPool;

use files::{files_delete, space_files_get, space_files_post};
use spaces::{spaces_delete, spaces_get, spaces_get_one, spaces_post, spaces_update};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    errors::{AppError, ErrorType, IntoAppError, init_logging},
    files::files_download,
};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    upload_path: String,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    init_logging();
    let upload_limit: usize = 1024 * 2 * 10_usize.pow(8);
    let allowed_origins: Vec<HeaderValue> = std::env::var("ALLOWED_ORIGINS")
        .map_err(|e| {
            AppError::new(
                ErrorType::Configuration("ALLOWED_ORIGINS required.".into()),
                e.into(),
            )
        })?
        .split(",")
        .map(|origin| {
            let trimmed = origin.trim();
            trimmed
                .parse::<HeaderValue>()
                .context(format!(
                    "Failed to parse origin {} as valid HTTP header value",
                    trimmed
                ))
                .map_err(|e| {
                    AppError::new(ErrorType::Configuration("CORS Origin invalid.".into()), e)
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL not set")
        .map_err(|e| {
            AppError::new(
                ErrorType::Configuration("DATABASE_URL must be set".into()),
                e,
            )
        })?;
    let upload_path = std::env::var("UPLOAD_PATH")
        .context("UPLOAD_PATH not set")
        .map_err(|e| {
            AppError::new(
                ErrorType::Configuration("UPLOAD_PATH must be set".into()),
                e,
            )
        })?;
    let upload_path_exists = Path::new(&upload_path).exists();
    if !upload_path_exists {
        panic!("The specified upload path doesnt exist!")
    }

    let pool = PgPool::connect(&database_url).await.into_db_error()?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .into_db_error()?;
    let state = AppState { pool, upload_path };

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(allowed_origins);

    let router_spaces = Router::new()
        .route("/", get(spaces_get).post(spaces_post))
        .route(
            "/{space_id}",
            get(spaces_get_one)
                .patch(spaces_update)
                .delete(spaces_delete),
        )
        .route(
            "/{space_id}/files",
            get(space_files_get)
                .post(space_files_post)
                // 200MB upload limit
                .layer(DefaultBodyLimit::max(upload_limit)),
        );

    let router_files = Router::new()
        .route("/{file_id}/download", get(files_download))
        .route("/{file_id}", delete(files_delete));

    let app = Router::new()
        .route("/health", get(|| async { "spaces up and running!" }))
        .nest("/api/spaces", router_spaces)
        .nest("/api/files", router_files)
        .layer(cors)
        .with_state(state);

    let address = "0.0.0.0:6570";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .into_internal_error()?;
    axum::serve(listener, app).await.into_internal_error()?;

    Ok(())
}
