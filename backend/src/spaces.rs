use axum::{
    Json, debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

use crate::{
    AppState,
    errors::{AppError, ErrorType, IntoAppError, ResponseError},
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
    // TODO access code filtering
    let rec: Vec<Space> = sqlx::query_as!(
        Space,
        "SELECT * FROM spaces WHERE is_public IS TRUE ORDER BY created_at DESC"
    )
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
    .await
    .into_db_error()?;

    Ok(Json::from(rec))
}

#[derive(Deserialize)]
pub struct CodeQuery {
    pub access_code: Option<String>,
}
#[debug_handler()]
pub async fn spaces_get_one(
    Path(space_id): Path<String>,
    Query(CodeQuery { access_code }): Query<CodeQuery>,
    State(AppState {
        pool,
        upload_path: _,
    }): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?;
    let Some(space) = rec else {
        return Err(AppError::new(
            ErrorType::NotFound("Space does not exist".to_string()),
            anyhow::anyhow!("Space does not exist"),
        ));
    };
    if space
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }

    Ok((StatusCode::OK, Json::from(space)))
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
    Query(CodeQuery { access_code }): Query<CodeQuery>,
    Json(payload): Json<UpdateSpaceRequest>,
) -> Result<impl IntoResponse, AppError> {
    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?;
    let Some(space) = rec else {
        return Err(AppError::new(
            ErrorType::NotFound("Space does not exist".to_string()),
            anyhow::anyhow!("Space does not exist"),
        ));
    };

    if space
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }

    let rec = sqlx::query_as!(
        Space,
        r#"
        UPDATE spaces
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_public = COALESCE($4, is_public),
            access_code = COALESCE($5, access_code)
        WHERE id = $1 AND (access_code = $5 OR access_code IS NULL)
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
    Query(CodeQuery { access_code }): Query<CodeQuery>,
) -> Result<impl IntoResponse, AppError> {
    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?;
    let Some(space) = rec else {
        return Err(AppError::new(
            ErrorType::NotFound("Space does not exist".to_string()),
            anyhow::anyhow!("Space does not exist"),
        ));
    };

    if space
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }
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
