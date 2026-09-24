use axum::{
    extract::{multipart::Field, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    entities::bug_reports,
    error::{AppError, AppResult},
    models::BugReportSubmitResponse,
    services::{Mailer, R2Service},
    state::AppState,
};

const MAX_TITLE: usize = 1024;
const MAX_VERSION: usize = 256;
const MAX_CONTENT: usize = 64 * 1024;
const MAX_EMAIL: usize = 1024;
const MAX_FILE: usize = 10 * 1024 * 1024;

/// Reads one multipart field into memory with a hard size cap.
async fn read_limited(mut field: Field<'_>, limit: usize) -> AppResult<Vec<u8>> {
    let mut data = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| AppError::BadRequest(format!("read multipart field: {}", e)))?
    {
        if data.len() + chunk.len() > limit {
            return Err(AppError::BadRequest("form field too large".to_string()));
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

fn read_text(value: Vec<u8>) -> AppResult<String> {
    String::from_utf8(value).map_err(|_| AppError::BadRequest("form fields must be UTF-8".to_string()))
}

/// Keeps a filename usable inside an R2 key and a download name.
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' })
        .collect();
    if cleaned.is_empty() {
        "file".to_string()
    } else {
        cleaned
    }
}

struct Attachment {
    filename: String,
    data: Vec<u8>,
}

/// Public bug-report submission (multipart form). Required text fields:
/// title, version, content. Optional: email, screenshot (image), log file.
/// The hidden `website` field is a spam honeypot: bots that fill it get a
/// fake success and nothing is stored.
pub async fn submit_bug_report(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<BugReportSubmitResponse>)> {
    let mut title = String::new();
    let mut app_version = String::new();
    let mut content = String::new();
    let mut email: Option<String> = None;
    let mut honeypot = String::new();
    let mut screenshot: Option<Attachment> = None;
    let mut log: Option<Attachment> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart body: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title" => title = read_text(read_limited(field, MAX_TITLE).await?)?.trim().to_string(),
            "version" => app_version = read_text(read_limited(field, MAX_VERSION).await?)?.trim().to_string(),
            "content" => content = read_text(read_limited(field, MAX_CONTENT).await?)?.trim().to_string(),
            "email" => {
                let value = read_text(read_limited(field, MAX_EMAIL).await?)?.trim().to_string();
                if !value.is_empty() {
                    email = Some(value);
                }
            }
            "website" => honeypot = read_text(read_limited(field, MAX_EMAIL).await?)?.trim().to_string(),
            "screenshot" | "log" => {
                let filename = field.file_name().unwrap_or("").to_string();
                if filename.is_empty() {
                    // Browsers always send the part; an empty filename means
                    // the user did not pick a file.
                    continue;
                }
                if name == "screenshot" {
                    match field.content_type() {
                        Some(ct) if ct.starts_with("image/") => {}
                        _ => return Err(AppError::BadRequest("screenshot must be an image".to_string())),
                    }
                }
                let data = read_limited(field, MAX_FILE).await?;
                let attachment = Attachment {
                    filename: sanitize_filename(&filename),
                    data,
                };
                if name == "screenshot" {
                    screenshot = Some(attachment);
                } else {
                    log = Some(attachment);
                }
            }
            _ => {}
        }
    }

    if !honeypot.is_empty() {
        return Ok((
            StatusCode::CREATED,
            Json(BugReportSubmitResponse {
                id: Uuid::new_v4(),
                message: "bug report received".to_string(),
            }),
        ));
    }

    if title.is_empty() || app_version.is_empty() || content.is_empty() {
        return Err(AppError::BadRequest(
            "title, version and content are required".to_string(),
        ));
    }
    if let Some(ref addr) = email {
        if !addr.contains('@') {
            return Err(AppError::BadRequest("invalid email address".to_string()));
        }
    }

    let id = Uuid::new_v4();
    let report = bug_reports::ActiveModel {
        id: Set(id),
        title: Set(title),
        app_version: Set(app_version),
        content: Set(content),
        email: Set(email),
        screenshot_key: Set(None),
        screenshot_filename: Set(None),
        log_key: Set(None),
        log_filename: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    };
    report.insert(&state.db).await?;

    // Attachments are best-effort: a failed upload must not lose the report.
    let r2 = R2Service::new(state.s3.clone(), &state.config.r2);
    let mut screenshot_key = None;
    let mut log_key = None;

    for (kind, attachment) in [("screenshot", screenshot), ("log", log)] {
        let Some(att) = attachment else { continue };
        let key = format!("bug-reports/{}/{}-{}", id, kind, att.filename);
        match r2
            .upload_streaming(&key, att.data.len() as i64, att.data.into())
            .await
        {
            Ok(_) => {
                if kind == "screenshot" {
                    screenshot_key = Some((key, att.filename));
                } else {
                    log_key = Some((key, att.filename));
                }
            }
            Err(e) => {
                tracing::error!("bug report {}: {} upload failed: {}", id, kind, e);
            }
        }
    }

    if screenshot_key.is_some() || log_key.is_some() {
        bug_reports::Entity::update(bug_reports::ActiveModel {
            id: Set(id),
            screenshot_key: Set(screenshot_key.as_ref().map(|(k, _)| k.clone())),
            screenshot_filename: Set(screenshot_key.as_ref().map(|(_, f)| f.clone())),
            log_key: Set(log_key.as_ref().map(|(k, _)| k.clone())),
            log_filename: Set(log_key.as_ref().map(|(_, f)| f.clone())),
            ..Default::default()
        })
        .exec(&state.db)
        .await?;
    }

    let stored = bug_reports::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Internal("bug report disappeared".to_string()))?;

    // Email is a notification on top of the stored report; never fail the
    // submission because mail delivery failed. Attachment links are permanent
    // backend URLs (they re-sign on each access) because the recipient is a
    // mailing list whose members may read the mail long after it was sent.
    match Mailer::from_config(&state.config.smtp) {
        Ok(Some(mailer)) => {
            let mut links = Vec::new();
            match state.config.server.public_url.as_ref().map(|b| b.trim_end_matches('/')) {
                Some(base) => {
                    if stored.screenshot_key.is_some() {
                        links.push(format!("{}/api/v1/bug-reports/{}/files/screenshot", base, id));
                    }
                    if stored.log_key.is_some() {
                        links.push(format!("{}/api/v1/bug-reports/{}/files/log", base, id));
                    }
                }
                None => {
                    if stored.screenshot_key.is_some() || stored.log_key.is_some() {
                        tracing::warn!(
                            "bug report {}: attachments stored but APP__SERVER__PUBLIC_URL is not set; email will not include attachment links",
                            id
                        );
                    }
                }
            }
            if let Err(e) = mailer.send_bug_report(&stored, &links).await {
                tracing::error!("bug report {}: failed to send notification email: {}", id, e);
            }
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("bug report {}: smtp misconfigured: {}", id, e);
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(BugReportSubmitResponse {
            id,
            message: "bug report received".to_string(),
        }),
    ))
}

/// Permanent attachment URLs for bug reports (used in notification emails).
/// Each hit signs a fresh R2 URL and redirects to it, so the link itself
/// never expires. Unauthenticated on purpose: the report UUID acts as the
/// bearer secret.
pub async fn get_bug_report_attachment(
    State(state): State<AppState>,
    Path((id, kind)): Path<(Uuid, String)>,
) -> AppResult<impl IntoResponse> {
    let report = bug_reports::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("bug report not found".to_string()))?;

    let key = match kind.as_str() {
        "screenshot" => report.screenshot_key,
        "log" => report.log_key,
        _ => None,
    }
    .ok_or_else(|| AppError::NotFound("attachment not found".to_string()))?;

    let r2 = R2Service::new(state.s3.clone(), &state.config.r2);
    let url = r2.generate_presigned_url(&key, Duration::from_secs(300)).await?;
    Ok(Redirect::temporary(&url))
}
