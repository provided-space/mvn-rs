use actix_web::{get, HttpResponse, put, Responder, web};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};

use crate::AppState;
use crate::coordinates::Parser;
use crate::model::maven::File;

fn is_metadata(uri: &str) -> Result<bool, regex::Error> {
    let pattern = match regex::Regex::new(".*/maven-metadata.xml(.*)") {
        Ok(pattern) => pattern,
        Err(err) => return Err(err),
    };
    return Ok(pattern.is_match(uri));
}

#[put("{path:.*}")]
async fn deploy(path: web::Path<String>, app_state: web::Data<AppState>, authentication: web::Header<Authorization<Basic>>, payload: web::Bytes) -> impl Responder {
    let uri = path.into_inner();
    match is_metadata(uri.as_ref()) {
        Ok(true) => return deploy_metadata(uri, app_state, authentication, payload).await,
        Err(_) => return HttpResponse::InternalServerError().into(),
        _ => {}
    };

    let coordinates = match Parser::parse_to_file(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().into(),
    };

    let authentication = authentication.as_ref();
    let result = app_state.authenticator.authenticate(authentication, &app_state.pool).await;
    if !result.is_success() {
        return result.as_response();
    }

    let file_path = format!("{}/{}", app_state.config.file_storage, uuid::Uuid::new_v4());
    if app_state.files.write(&file_path, payload.as_ref()).is_err() {
        return HttpResponse::InternalServerError().into();
    }

    let version = match app_state.version_repository.get_or_create_version(&app_state.pool, coordinates.clone().to_version()).await {
        Ok(Some(version)) => version,
        Ok(None) => return HttpResponse::InternalServerError().into(),
        Err(_) => return HttpResponse::Conflict().into(), // TODO: A conflict occurs if an unique key constraint is violated. Otherwise we should return an internal server error
    };

    let result = sqlx::query!("INSERT INTO maven_file (version_id, name, uri, path) VALUES (?, ?, ?, ?)", version.id, coordinates.file, uri, file_path).execute(&app_state.pool).await;
    if result.is_err() {
        // TODO: A conflict occurs if an unique key constraint is violated. Otherwise we should return an internal server error
        return HttpResponse::Conflict().into();
    }

    return HttpResponse::Ok().into();
}

// For now, we assume that we're just getting metadata for artifacts
async fn deploy_metadata(uri: String, app_state: web::Data<AppState>, authentication: web::Header<Authorization<Basic>>, payload: web::Bytes) -> HttpResponse {
    let coordinates = match Parser::parse_to_version(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().into(),
    };

    let authentication = authentication.as_ref();
    let result = app_state.authenticator.authenticate(authentication, &app_state.pool).await;
    if !result.is_success() {
        return result.as_response();
    }

    let file_path = format!("{}/{}", app_state.config.file_storage, uuid::Uuid::new_v4());
    if app_state.files.write(&file_path, payload.as_ref()).is_err() {
        return HttpResponse::InternalServerError().into();
    }

    let result = sqlx::query!("INSERT INTO maven_file (name, uri, path) VALUES (?, ?, ?)", coordinates.artifact, uri, file_path).execute(&app_state.pool).await;
    if result.is_err() {
        // TODO: A conflict occurs if an unique key constraint is violated. Otherwise we should return an internal server error
        return HttpResponse::Conflict().into();
    }

    return HttpResponse::Ok().into();
}

#[get("{path:.*}")]
async fn read(path: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let uri = path.into_inner();
    let coordinates = match Parser::parse_to_file(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().into(),
    };

    let file_result = sqlx::query_as!(File, "SELECT file.id, file.version_id, file.name, file.uri, file.path FROM maven_file file WHERE file.uri = ?", uri).fetch_optional(&app_state.pool).await;
    if file_result.is_err() {
        return HttpResponse::InternalServerError().into();
    }

    let file = match file_result.unwrap() {
        Some(file) => file,
        None => return HttpResponse::NotFound().into(),
    };

    return match app_state.files.read(&file.path) {
        Ok(contents) => HttpResponse::Ok().body(contents),
        Err(_) => HttpResponse::InternalServerError().into(),
    };
}
