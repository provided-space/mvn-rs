use actix_web::{App, get, HttpResponse, HttpServer, main, Responder, web};
use sqlx::{MySql, Pool};
use sqlx::mysql::MySqlPoolOptions;

use crate::authentication::authenticator::Authenticator;
use crate::config::ApplicationConfig;
use crate::file_service::FileService;
use crate::repository::maven::{ArtifactRepository, VersionRepository};

mod config;
mod model;
mod route;
mod authentication;
mod coordinates;
mod repository;
mod file_service;

#[derive(Clone)]
struct AppState {
    pool: Pool<MySql>,
    config: ApplicationConfig,
    authenticator: Authenticator,
    version_repository: VersionRepository,
    artifact_repository: ArtifactRepository,
    files: FileService,
}

#[main]
async fn main() -> std::io::Result<()> {
    let config = config::parse();

    println!("Connecting to database");
    let pool = match MySqlPoolOptions::new().connect(&config.database.url.as_str()).await {
        Ok(pool) => pool,
        Err(error) => panic!("Failed to connect to the database. {error}"),
    };

    println!("Running migrations");
    if let Err(error) = sqlx::migrate!("./migrations").run(&pool).await {
        panic!("Failed to run migrations. {error}");
    }

    let artifact_repository = ArtifactRepository::new();
    let app_state = AppState {
        pool,
        config: config.clone(),
        authenticator: Authenticator::new(config.authentication.secret),
        version_repository: VersionRepository::new(artifact_repository.clone()),
        artifact_repository,
        files: FileService::new(),
    };
    let webserver = config.webserver;
    println!("Starting webserver on {}:{}", webserver.host, webserver.port);

    return HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(favicon)
            .service(route::read)
            .service(route::deploy)
    })
        .bind(webserver.to_address())?
        .run()
        .await;
}

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    return HttpResponse::Ok().body(&include_bytes!("../assets/favicon.ico")[..]);
}
