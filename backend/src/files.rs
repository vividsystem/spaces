use std::fs::remove_file;

use anyhow::anyhow;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::serde::rfc3339 as rfc3339_mod;
use tokio::{fs::File, io::AsyncWriteExt};

use axum::{
    Json,
    body::Body,
    debug_handler,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};
use tokio_util::io::ReaderStream;

use crate::{
    AppState,
    errors::{AppError, ErrorType, IntoAppError},
    spaces::Space,
};

fn serialize_opt<S: Serializer>(opt: &Option<OffsetDateTime>, s: S) -> Result<S::Ok, S::Error> {
    match opt {
        Some(dt) => rfc3339_mod::serialize(dt, s),
        None => s.serialize_none(),
    }
}

#[derive(Serialize)]
pub struct SpaceFile {
    id: String,
    space_id: String,
    original_filename: String,
    file_size_bytes: i64,
    mime_type: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    upload_date: OffsetDateTime,
    #[serde(serialize_with = "serialize_opt")]
    last_accessed: Option<OffsetDateTime>,
    download_count: i32,
    checksum: String,
}

#[debug_handler()]
pub async fn space_files_post(
    State(AppState { pool, upload_path }): State<AppState>,
    Path(space_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Vec<SpaceFile>>, AppError> {
    // TODO: change 2MB file upload limit
    let mut files: Vec<SpaceFile> = Vec::new();

    // check if space exists
    let rec = sqlx::query_as!(Space, "SELECT * from spaces where id = $1", space_id)
        .fetch_optional(&pool)
        .await
        .into_db_error()?
        .ok_or_else(|| {
            AppError::new(
                ErrorType::NotFound("Space not found".into()),
                anyhow!("Couldn't find requested Space"),
            )
        })?;
    while let Some(field) = multipart.next_field().await.into_validation_error()? {
        let old_filename: Option<String> = field.file_name().map(|s| s.to_string());

        let filetype = field
            .content_type()
            .expect("Content-Type should be set")
            .to_string();

        let data = field.bytes().await.into_validation_error()?;
        let file_size_bytes = data.len() as i64;
        let checksum = format!("{:x}", Sha256::digest(&data));

        let id = uuid::Uuid::new_v4();

        let filepath = std::path::Path::new(&upload_path).join(&checksum);
        if filepath.exists() {
            //handle this
        } else {
            let mut file = File::create_new(&filepath)
                .await
                .expect("Filename should be unique and therefore not existant on creation!");

            file.write_all(&data).await.into_internal_error()?;
        }

        let file_rec = sqlx::query_as!(
            SpaceFile,
            r#"INSERT INTO files (id, space_id, original_filename, file_size_bytes, checksum, mime_type) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
            id.to_string(),
            rec.id,
            old_filename,
            file_size_bytes,
            checksum,
            filetype
        ).fetch_one(&pool).await.into_db_error()?;
        files.push(file_rec);
    }

    let total_file_sizes: i64 = files.iter().map(|file| file.file_size_bytes).sum();

    sqlx::query!(
        r#"UPDATE spaces SET total_size_used_bytes = total_size_used_bytes + $2 WHERE id = $1"#,
        space_id,
        total_file_sizes
    )
    .execute(&pool)
    .await
    .into_db_error()?;

    Ok(Json::from(files))
}

#[debug_handler()]
pub async fn space_files_get(
    State(AppState { pool, .. }): State<AppState>,
    Path(space_id): Path<String>,
) -> Result<Json<Vec<SpaceFile>>, AppError> {
    // let rec = sqlx::query_as!(Space, "SELECT * from spaces where id = $1", space_id)
    //     .fetch_optional(&pool)
    //     .await?
    //     .expect("Space doesn't exist!");

    let files = sqlx::query_as!(
        SpaceFile,
        r"SELECT * from files where space_id = $1",
        space_id,
    )
    .fetch_all(&pool)
    .await
    .into_db_error()?;

    Ok(Json::from(files))
}

#[debug_handler()]
pub async fn files_download(
    State(AppState { pool, upload_path }): State<AppState>,
    Path(file_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_meta = sqlx::query_as!(SpaceFile, r"SELECT * from files where id = $1", file_id,)
        .fetch_optional(&pool)
        .await
        .into_db_error()?
        .ok_or_else(|| {
            AppError::new(
                ErrorType::Validation("File not found".into()),
                anyhow!("Requested file not stored in database"),
            )
        })?;

    let mut headers = HeaderMap::new();

    if let Some(mime_type) = file_meta.mime_type {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&mime_type).into_internal_error()?,
        );
    }

    let content_disposition = format!("attachment; filename=\"{}\"", file_meta.original_filename);
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition).into_internal_error()?,
    );

    let filepath = std::path::Path::new(&upload_path).join(file_meta.checksum);

    let file = File::open(filepath).await.into_internal_error()?;

    sqlx::query!(
        r#"UPDATE files SET download_count = download_count + 1 WHERE id = $1"#,
        file_meta.id,
    )
    .execute(&pool)
    .await
    .into_db_error()?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((headers, body))
}

#[debug_handler()]
pub async fn files_delete(
    State(AppState { pool, upload_path }): State<AppState>,
    Path(file_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_meta = sqlx::query_as!(
        SpaceFile,
        r#"DELETE from files where id = $1 RETURNING *"#,
        file_id,
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?
    .ok_or_else(|| {
        AppError::new(
            ErrorType::Validation("File not found".into()),
            anyhow!("Requested file not stored in database"),
        )
    })?;

    sqlx::query!(
        r#"UPDATE spaces SET total_size_used_bytes = GREATEST(0, total_size_used_bytes - $2) WHERE id = $1"#,
        file_meta.space_id,
        file_meta.file_size_bytes
    ).execute(&pool).await.into_db_error()?;

    let other_files = sqlx::query_as!(
        SpaceFile,
        r#"SELECT * from files where checksum = $1"#,
        file_meta.checksum
    )
    .fetch_all(&pool)
    .await
    .into_db_error()?;

    let filepath = std::path::Path::new(&upload_path).join(&file_meta.checksum);
    if other_files.len() == 0 {
        remove_file(filepath).map_err(|e| {
            AppError::new(
                ErrorType::Internal("An error occured removing the file".into()),
                e.into(),
            )
        })?;
    }

    Ok(Json::from(file_meta))
}
