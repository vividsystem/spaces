use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

use crate::{
    AppState,
    errors::{AppError, IntoAppError},
};

#[derive(Debug, Serialize, FromRow)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,

    pub is_public: bool,
    pub access_code: Option<String>,
    pub total_size_used_bytes: i64,
}

#[debug_handler()]
pub async fn spaces_get(
    State(AppState {
        pool,
        upload_path: _,
    }): State<AppState>,
) -> Result<Json<Vec<Space>>, AppError> {
    let rec: Vec<Space> = sqlx::query_as!(Space, "SELECT * FROM spaces ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .into_db_error()?;

    Ok(Json::from(rec))
}

#[derive(Deserialize, FromRow)]
pub struct CreateSpaceRequest {
    name: String,
    description: Option<String>,
    #[sqlx(default)]
    is_public: Option<bool>,
    access_code: Option<String>,
}

#[debug_handler()]
pub async fn spaces_post(
    State(AppState {
        pool,
        upload_path: _,
    }): State<AppState>,
    Json(payload): Json<CreateSpaceRequest>,
) -> Result<Json<Space>, AppError> {
    let id = uuid::Uuid::new_v4().to_string();

    let rec = sqlx::query_as!(
        Space,
        "INSERT INTO spaces (id, name, description, is_public, access_code) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        id,
        payload.name,
        payload.description,
        payload.is_public.unwrap_or(false),
        payload.access_code
    )
    .fetch_one(&pool)
    .await.into_db_error()?;

    Ok(Json::from(rec))
}

#[debug_handler()]
pub async fn spaces_get_one(
    Path(space_id): Path<String>,
    State(AppState {
        pool,
        upload_path: _,
    }): State<AppState>,
) -> Result<Json<Option<Space>>, AppError> {
    let rec = sqlx::query_as!(Space, "SELECT * FROM spaces WHERE id = $1", space_id)
        .fetch_optional(&pool)
        .await
        .into_db_error()?;

    Ok(Json::from(rec))
}

#[derive(Deserialize, FromRow)]
pub struct UpdateSpaceRequest {
    name: String,
    description: Option<String>,
    #[sqlx(default)]
    is_public: Option<bool>,
    access_code: Option<String>,
}

#[debug_handler()]
pub async fn spaces_update(
    State(AppState {
        pool,
        upload_path: _,
    }): State<AppState>,

    Path(space_id): Path<String>,
    Json(payload): Json<UpdateSpaceRequest>,
) -> Result<Json<Space>, AppError> {
    let rec = sqlx::query_as!(
        Space,
        r#"
        UPDATE spaces
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_public = COALESCE($4, is_public),
            access_code = COALESCE($5, access_code)
        WHERE id = $1
        RETURNING *;
        "#,
        space_id,
        payload.name,
        payload.description,
        payload.is_public,
        payload.access_code
    )
    .fetch_one(&pool)
    .await
    .into_db_error()?;

    Ok(Json::from(rec))
}

#[debug_handler()]
pub async fn spaces_delete(
    State(AppState {
        pool,
        upload_path: _,
    }): State<AppState>,
    Path(space_id): Path<String>,
) -> Result<Json<Option<Space>>, AppError> {
    let rec = sqlx::query_as!(
        Space,
        r#"
        DELETE FROM spaces WHERE id = $1
        RETURNING *;
        "#,
        space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?;

    Ok(Json::from(rec))
}
