use actix_web::{get, HttpResponse, put, Responder, web};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};

use crate::AppState;
use crate::authentication::{ArtifactPermission, AuthenticationResult};
use crate::coordinates::{Artifact as ArtifactDTO, Parser};
use crate::model::maven::File;

fn is_metadata(uri: &str) -> Result<bool, regex::Error> {
    let pattern = match regex::Regex::new(".*/maven-metadata.xml(.*)") {
        Ok(pattern) => pattern,
        Err(err) => return Err(err),
    };
    return Ok(pattern.is_match(uri));
}

#[put("{path:.*}")]
pub async fn deploy(path: web::Path<String>, app_state: web::Data<AppState>, authentication: web::Header<Authorization<Basic>>, payload: web::Bytes) -> impl Responder {
    let uri = path.into_inner();
    match is_metadata(uri.as_ref()) {
        Ok(true) => return deploy_metadata(uri, app_state, authentication, payload).await,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to check if the file is metadata."),
        _ => {}
    };

    let coordinates = match Parser::parse_to_file(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().body("Coordinates could not be parsed."),
    };

    let authentication = authentication.as_ref();
    let access_token = match app_state.authenticator.authenticate(authentication, &app_state.pool).await {
        AuthenticationResult::Success(access_token) => access_token,
        result => return result.as_response(),
    };

    let version = match app_state.version_repository.get_or_create_version(&app_state.pool, coordinates.clone().to_version(), access_token).await {
        Ok(Some(version)) => version,
        Ok(None) => return HttpResponse::InternalServerError().body("No applicable artifact version could be obtained."),
        Err(sqlx::Error::Database(err)) => {
            if err.is_unique_violation() {
                return HttpResponse::Conflict().body("This artifact version would have led to a duplicate.");
            }
            return HttpResponse::InternalServerError().body("Failed to handle the specified version due to a database error.");
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to handle the specified version."),
    };

    let file_path = format!("{}/{}", app_state.config.file_storage, uuid::Uuid::new_v4());
    let result = sqlx::query!("INSERT INTO maven_file (version_id, name, uri, path) VALUES (?, ?, ?, ?)", version.id, coordinates.file, uri, file_path).execute(&app_state.pool).await;
    match result {
        Err(sqlx::Error::Database(err)) => {
            if err.is_unique_violation() {
                return HttpResponse::Conflict().body("The given file already exists and cannot be overridden.");
            }
            return HttpResponse::InternalServerError().body("Failed to persist file due to a database error.");
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to persist file."),
        _ => {},
    };

    if app_state.files.write(&file_path, payload.as_ref()).is_err() {
        return HttpResponse::InternalServerError().body("Failed to write file contents.");
    }
    return HttpResponse::Ok().into();
}

// For now, we assume that we're just getting metadata for artifacts
async fn deploy_metadata(uri: String, app_state: web::Data<AppState>, authentication: web::Header<Authorization<Basic>>, payload: web::Bytes) -> HttpResponse {
    let coordinates = match Parser::parse_to_version(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().body("Coordinates could not be parsed."),
    };

    let authentication = authentication.as_ref();
    let access_token = match app_state.authenticator.authenticate(authentication, &app_state.pool).await {
        AuthenticationResult::Success(access_token) => access_token,
        result => return result.as_response(),
    };

    match app_state.artifact_repository.get_writable_artifact(&app_state.pool, coordinates.clone().to_artifact(), access_token).await {
        Err(_) => return HttpResponse::InternalServerError().body("Failed to obtain permission for writing to this scope."),
        Ok(None) => return HttpResponse::Forbidden().body("Identity is not permitted to write to this scope."),
        _ => {},
    };

    let file_result = sqlx::query_as!(File, "SELECT file.id, file.version_id, file.name, file.uri, file.path FROM maven_file file WHERE file.uri = ?", uri).fetch_optional(&app_state.pool).await;
    match file_result {
        Ok(Some(file)) => {
            if app_state.files.write(&file.path, payload.as_ref()).is_err() {
                return HttpResponse::InternalServerError().body("Failed to write file contents.");
            }
            return HttpResponse::Ok().into();
        },
        Err(sqlx::Error::Database(_err)) => return HttpResponse::InternalServerError().body("Failed to persist metadata."),
        _ => {},
    };

    let file_path = format!("{}/{}", app_state.config.file_storage, uuid::Uuid::new_v4());
    let result = sqlx::query!("INSERT INTO maven_file (name, uri, path) VALUES (?, ?, ?)", coordinates.artifact, uri, file_path).execute(&app_state.pool).await;
    match result {
        Err(sqlx::Error::Database(err)) => {
            if err.is_unique_violation() {
                return HttpResponse::Conflict().body("The given metadata already exists.");
            }
            return HttpResponse::InternalServerError().body("Failed to persist metadata due to a database error.");
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to persist metadata."),
        _ => {},
    };

    if app_state.files.write(&file_path, payload.as_ref()).is_err() {
        return HttpResponse::InternalServerError().body("Failed to write file contents.");
    }
    return HttpResponse::Ok().into();
}

#[get("{path:.*}")]
pub async fn read(path: web::Path<String>, app_state: web::Data<AppState>, authentication: Option<web::Header<Authorization<Basic>>>) -> impl Responder {
    let uri = path.into_inner();
    match is_metadata(uri.as_ref()) {
        Ok(true) => return read_metadata(uri, app_state, authentication).await,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to check if the file is metadata."),
        _ => {}
    };

    let coordinates = match Parser::parse_to_file(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().body("Coordinates could not be parsed."),
    };

    return read_file(uri, app_state, authentication, coordinates.to_version().to_artifact()).await;
}

// For now, we assume that we're just serving metadata for artifacts
async fn read_metadata(uri: String, app_state: web::Data<AppState>, authentication: Option<web::Header<Authorization<Basic>>>) -> HttpResponse {
    let coordinates = match Parser::parse_to_version(uri.as_str()) {
        Ok(coordinates) => coordinates,
        Err(_) => return HttpResponse::BadRequest().body("Coordinates could not be parsed."),
    };

    return read_file(uri, app_state, authentication, coordinates.to_artifact()).await;
}

async fn read_file(uri: String, app_state: web::Data<AppState>, authentication: Option<web::Header<Authorization<Basic>>>, artifact: ArtifactDTO) -> HttpResponse {
    match app_state.artifact_authenticator.authenticate_read(artifact, authentication, &app_state.pool).await {
        ArtifactPermission::Granted => {},
        permission => return permission.as_response(),
    };

    let file_result = sqlx::query_as!(File, "SELECT file.id, file.version_id, file.name, file.uri, file.path FROM maven_file file WHERE file.uri = ?", uri).fetch_optional(&app_state.pool).await;
    let file = match file_result {
        Ok(Some(file)) => file,
        Ok(None) => return HttpResponse::NotFound().body("The requested file does not exist in this registry."),
        Err(_) => return HttpResponse::InternalServerError().body("Querying for the requested file failed."),
    };

    return match app_state.files.read(&file.path) {
        Ok(contents) => HttpResponse::Ok().body(contents),
        Err(_) => HttpResponse::InternalServerError().body("File contents could not be obtained."),
    };
}
