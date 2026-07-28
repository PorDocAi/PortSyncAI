use std::path::{Path as FilePath, PathBuf};

use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryOrder};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use vespera::multipart::{FieldData, TypedMultipart};

use crate::models::cargo_documents::{
    self, CargoDocumentType, Entity as CargoDocuments, FileFormat,
};
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(vespera::Multipart, vespera::Schema)]
#[try_from_multipart(strict)]
pub struct UploadCargoDocumentRequest {
    /// BL | DGD
    pub document_type: String,
    #[form_data(limit = "20MiB")]
    pub file: FieldData<tempfile::NamedTempFile>,
}

#[derive(Serialize, vespera::Schema)]
pub struct CargoDocumentResponse {
    pub cargo_document_id: i64,
    pub document_type: String,
    pub file_format: String,
    pub file_url: String,
    pub original_file_name: Option<String>,
    pub content_type: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub uploaded_by_id: i64,
    pub created_at: String,
}

fn document_type_as_str(value: &CargoDocumentType) -> &'static str {
    match value {
        CargoDocumentType::Bl => "BL",
        CargoDocumentType::Dgd => "DGD",
    }
}

fn file_format_as_str(value: &FileFormat) -> &'static str {
    match value {
        FileFormat::Hwp => "HWP",
        FileFormat::Pdf => "PDF",
        FileFormat::Xlsx => "XLSX",
        FileFormat::Docx => "DOCX",
        FileFormat::Image => "IMAGE",
    }
}

impl From<cargo_documents::Model> for CargoDocumentResponse {
    fn from(model: cargo_documents::Model) -> Self {
        Self {
            cargo_document_id: model.cargo_document_id,
            document_type: document_type_as_str(&model.document_type).to_string(),
            file_format: file_format_as_str(&model.file_format).to_string(),
            file_url: model.file_url,
            original_file_name: model.original_file_name,
            content_type: model.content_type,
            file_size: model.file_size,
            file_hash: model.file_hash,
            uploaded_by_id: model.uploaded_by_id,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}

fn parse_document_type(value: &str) -> Result<CargoDocumentType, StatusCode> {
    match value.trim().to_ascii_uppercase().as_str() {
        "BL" => Ok(CargoDocumentType::Bl),
        "DGD" => Ok(CargoDocumentType::Dgd),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn parse_file_format(file_name: &str) -> Result<(FileFormat, String), StatusCode> {
    let extension = FilePath::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or(StatusCode::UNSUPPORTED_MEDIA_TYPE)?;

    let format = match extension.as_str() {
        "hwp" => FileFormat::Hwp,
        "pdf" => FileFormat::Pdf,
        "xlsx" => FileFormat::Xlsx,
        "docx" => FileFormat::Docx,
        "jpg" | "jpeg" | "png" => FileFormat::Image,
        _ => return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE),
    };
    Ok((format, extension))
}

async fn calculate_sha256(path: &FilePath) -> Result<String, StatusCode> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// B/L·DGD 원문 업로드. 파일은 로컬 스토리지에 UUID 이름으로 저장하고 메타데이터만 DB에 기록한다.
#[vespera::route(post, tags = ["cargo_documents"])]
pub async fn upload_cargo_document(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    TypedMultipart(req): TypedMultipart<UploadCargoDocumentRequest>,
) -> Result<(StatusCode, Json<CargoDocumentResponse>), StatusCode> {
    let document_type = parse_document_type(&req.document_type)?;
    let original_file_name = req
        .file
        .metadata
        .file_name
        .clone()
        .filter(|name| !name.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    if original_file_name.len() > 255 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let (file_format, extension) = parse_file_format(&original_file_name)?;
    let content_type = req.file.metadata.content_type.clone();
    if content_type.as_ref().is_some_and(|value| value.len() > 100) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let temporary_path = req.file.contents.path();
    let metadata = tokio::fs::metadata(temporary_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if metadata.len() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let file_size = i64::try_from(metadata.len()).map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let file_hash = calculate_sha256(temporary_path).await?;

    let storage_key = format!("cargo/{}.{}", Uuid::new_v4(), extension);
    let physical_path = PathBuf::from(&state.config.upload_dir).join(&storage_key);
    if let Some(parent) = physical_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    tokio::fs::copy(temporary_path, &physical_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active = cargo_documents::ActiveModel {
        document_type: Set(document_type),
        file_format: Set(file_format),
        file_url: Set(storage_key),
        original_file_name: Set(Some(original_file_name)),
        content_type: Set(content_type),
        file_size: Set(Some(file_size)),
        file_hash: Set(Some(file_hash)),
        uploaded_by_id: Set(claims.sub),
        ..Default::default()
    };
    let saved = match active.insert(&state.db).await {
        Ok(saved) => saved,
        Err(_) => {
            let _ = tokio::fs::remove_file(&physical_path).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(CargoDocumentResponse::from(saved)),
    ))
}

/// 업로드된 화물 문서 메타데이터 목록
#[vespera::route(get, tags = ["cargo_documents"])]
pub async fn list_cargo_documents(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<CargoDocumentResponse>>, StatusCode> {
    let rows = CargoDocuments::find()
        .order_by_desc(cargo_documents::Column::CargoDocumentId)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter().map(CargoDocumentResponse::from).collect(),
    ))
}

/// 업로드된 화물 문서 메타데이터 단건 조회
#[vespera::route(get, path = "/{id}", tags = ["cargo_documents"])]
pub async fn get_cargo_document(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CargoDocumentResponse>, StatusCode> {
    let row = CargoDocuments::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(CargoDocumentResponse::from(row)))
}
