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
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};
use tokio_util::io::ReaderStream;
use tracing::warn;

use crate::{
    AppState,
    errors::{AppError, ErrorType, IntoAppError},
    spaces::{CodeQuery, Space},
};

fn serialize_opt<S: Serializer>(opt: &Option<OffsetDateTime>, s: S) -> Result<S::Ok, S::Error> {
    match opt {
        Some(dt) => rfc3339_mod::serialize(dt, s),
        None => s.serialize_none(),
    }
}

#[derive(Serialize, Debug)]
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
    Query(CodeQuery { access_code }): Query<CodeQuery>,
    Path(space_id): Path<String>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Vec<SpaceFile>>, AppError> {
    println!("HANDLER HIT");
    println!("content-type: {:?}", headers.get("content-type"));
    println!("space_id: {:?}", space_id);
    // TODO: change 2MB file upload limit
    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?
    .ok_or_else(|| {
        AppError::new(
            ErrorType::NotFound("Space does not exist".to_string()),
            anyhow::anyhow!("Space does not exist"),
        )
    })?;

    if rec
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }

    let mut files: Vec<SpaceFile> = Vec::new();
    let mut new_bytes: i64 = 0;
    let mut n = 0;
    while let Some(field) = multipart.next_field().await.into_validation_error()? {
        n += 1;
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
            let file_rec = sqlx::query_as!(
                SpaceFile,
                r#"SELECT * FROM files WHERE checksum = $1"#,
                checksum
            )
            .fetch_optional(&pool)
            .await
            .into_db_error()?
            .ok_or_else(|| {
                AppError::new(
                    ErrorType::Internal("Something went wrong".to_string()),
                    anyhow::anyhow!(
                        "State mismatch between saved files and metadata for checksum {}",
                        checksum
                    ),
                )
            })?;
            files.push(file_rec);
        } else {
            let mut file = File::create_new(&filepath)
                .await
                .expect("Filename should be unique and therefore not existant on creation!");

            file.write_all(&data).await.into_internal_error()?;

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
            new_bytes += file_size_bytes;
            files.push(file_rec);
        }
    }
    if n == 0 {
        return Err(AppError::new(
            ErrorType::Validation("No files detected".to_string()),
            anyhow::anyhow!("No files/fields detected in multipart"),
        ));
    }

    sqlx::query!(
        r#"UPDATE spaces SET total_size_used_bytes = total_size_used_bytes + $2 WHERE id = $1"#,
        space_id,
        new_bytes
    )
    .execute(&pool)
    .await
    .into_db_error()?;

    Ok(Json::from(files))
}

#[debug_handler()]
pub async fn space_files_get(
    State(AppState { pool, .. }): State<AppState>,
    Query(CodeQuery { access_code }): Query<CodeQuery>,
    Path(space_id): Path<String>,
) -> Result<Json<Vec<SpaceFile>>, AppError> {
    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?
    .ok_or_else(|| {
        AppError::new(
            ErrorType::NotFound("Space does not exist".to_string()),
            anyhow::anyhow!("Space does not exist"),
        )
    })?;

    if rec
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }

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
    Query(CodeQuery { access_code }): Query<CodeQuery>,
    Path(file_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_meta = sqlx::query_as!(SpaceFile, r"SELECT * from files where id = $1", file_id)
        .fetch_optional(&pool)
        .await
        .into_db_error()?
        .ok_or_else(|| {
            AppError::new(
                ErrorType::Validation("File not found".into()),
                anyhow!("Requested file not stored in database"),
            )
        })?;

    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        file_meta.space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?
    .ok_or_else(|| {
        AppError::new(
            ErrorType::NotFound("Space does not exist".to_string()),
            anyhow::anyhow!("Space does not exist"),
        )
    })?;

    if rec
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
        && !rec.is_public
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }

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
    Query(CodeQuery { access_code }): Query<CodeQuery>,
    Path(file_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_rec = sqlx::query_as!(SpaceFile, r"SELECT * from files where id = $1", file_id)
        .fetch_optional(&pool)
        .await
        .into_db_error()?
        .ok_or_else(|| {
            AppError::new(
                ErrorType::NotFound("space does not exist".to_string()),
                anyhow::anyhow!("space does not exist"),
            )
        })?;

    let rec = sqlx::query_as!(
        Space,
        r#"
        SELECT * FROM spaces WHERE id = $1
        "#,
        file_rec.space_id
    )
    .fetch_optional(&pool)
    .await
    .into_db_error()?
    .ok_or_else(|| {
        AppError::new(
            ErrorType::NotFound("space does not exist".to_string()),
            anyhow::anyhow!("space does not exist"),
        )
    })?;

    if rec
        .access_code
        .clone()
        .is_some_and(|c| access_code.is_none_or(|ac| ac != c))
    {
        return Err(AppError::new(
            ErrorType::Authentication("Bad credentials".to_string()),
            anyhow::anyhow!("Bad credentials"),
        ));
    }
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
            ErrorType::NotFound("File not found".to_string()),
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
