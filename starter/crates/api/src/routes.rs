use std::sync::Arc;

use actix_web::{HttpResponse, Responder, web};
use domain::{NoteInput, SearchQuery};
use platform::Platform;
use serde_json::json;

use crate::ApiError;

pub struct AppState {
    pub platform: Arc<Platform>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz))
        .route("/readyz", web::get().to(readyz))
        .route("/v1/notes", web::post().to(create_note))
        .route("/v1/notes/search", web::post().to(search_notes))
        .route("/v1/notes/{id}", web::get().to(get_note))
        .route("/v1/notes/{id}", web::delete().to(delete_note));
}

async fn healthz() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}

async fn readyz(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    state.platform.ready().await?;
    Ok(HttpResponse::Ok().json(json!({"status": "ready"})))
}

async fn create_note(
    state: web::Data<AppState>,
    input: web::Json<NoteInput>,
) -> Result<HttpResponse, ApiError> {
    input.validate()?;
    let id = input.id.unwrap_or_else(new_id);
    state.platform.upsert(id, &input).await?;
    Ok(HttpResponse::Created().json(json!({"id": id})))
}

async fn get_note(
    state: web::Data<AppState>,
    id: web::Path<u64>,
) -> Result<HttpResponse, ApiError> {
    match state.platform.get(*id).await? {
        Some(note) => Ok(HttpResponse::Ok().json(note)),
        None => Err(ApiError::NotFound),
    }
}

async fn delete_note(
    state: web::Data<AppState>,
    id: web::Path<u64>,
) -> Result<HttpResponse, ApiError> {
    state.platform.delete(*id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn search_notes(
    state: web::Data<AppState>,
    query: web::Json<SearchQuery>,
) -> Result<HttpResponse, ApiError> {
    query.validate()?;
    let hits = state.platform.search(&query).await?;
    let count = hits.len();
    Ok(HttpResponse::Ok().json(json!({"count": count, "hits": hits})))
}

fn new_id() -> u64 {
    uuid::Uuid::now_v7().as_u128() as u64
}
