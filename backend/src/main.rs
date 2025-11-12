use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
};

mod files;
mod spaces;

use sqlx::PgPool;

use files::{files_delete, files_get, space_files_get, space_files_post};
use spaces::{spaces_delete, spaces_get, spaces_get_one, spaces_post, spaces_update};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    upload_path: String,
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let upload_path = std::env::var("UPLOAD_PATH").expect("UPLOAD_PATH must be set");
    let pool = PgPool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    let state = AppState { pool, upload_path };

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
            get(space_files_get).post(space_files_post),
        );

    let router_files = Router::new()
        .route("/{file_id}/download", get(files_get))
        .route("/{file_id}", delete(files_delete));

    let app = Router::new()
        .route("/health", get(|| async { "spaces up and running!" }))
        .nest("/api/spaces", router_spaces)
        .nest("/api/files", router_files)
        .with_state(state);

    let address = "0.0.0.0:6570";
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
