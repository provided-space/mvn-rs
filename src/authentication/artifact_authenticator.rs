use actix_web::web::Header;
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use sqlx::{MySql, Pool};
use crate::authentication::{ArtifactPermission, AuthenticationResult};
use crate::authentication::ArtifactPermission::{Granted, Unauthorized, Forbidden, Error};
use crate::authentication::authenticator::Authenticator;
use crate::coordinates::Artifact as ArtifactDTO;
use crate::repository::maven::artifact_repository::ArtifactRepository;

#[derive(Clone)]
pub struct ArtifactAuthenticator {
    artifact_repository: ArtifactRepository,
    authenticator: Authenticator,
}

impl ArtifactAuthenticator {
    pub fn new(artifact_repository: ArtifactRepository, authenticator: Authenticator) -> ArtifactAuthenticator {
        return ArtifactAuthenticator { artifact_repository, authenticator };
    }

    pub async fn authenticate_read(&self, artifact: ArtifactDTO, authentication: Option<Header<Authorization<Basic>>>, pool: &Pool<MySql>) -> ArtifactPermission {
        let artifact = match self.artifact_repository.get_artifact(pool, artifact).await {
            Ok(Some(artifact)) => artifact,
            Ok(None) => return Error("The requested artifact does not exist in this registry.".to_owned()),
            Err(_) => return Error("".to_owned()),
        };
        if artifact.public {
            return Granted;
        }

        return match authentication {
            None => Unauthorized,
            Some(authentication) => {
                let access_token = match self.authenticator.authenticate(authentication.as_ref(), pool).await {
                    AuthenticationResult::Success(access_token) => access_token,
                    AuthenticationResult::Unauthorized => return Unauthorized,
                    AuthenticationResult::Forbidden => return Forbidden,
                    AuthenticationResult::Error(reason) => return Error(reason),
                };
                match self.artifact_repository.can_read(pool, artifact, access_token).await {
                    Ok(true) => Granted,
                    Ok(false) => Forbidden,
                    Err(_) => Error("Failed to check if identity is permitted to read from the requested artifact.".to_owned()),
                }
            }
        };
    }
}