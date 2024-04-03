use sqlx::{Error, MySql, Pool};

use crate::model::maven::{Version};
use crate::coordinates::Version as VersionDTO;
use crate::model::AccessToken;
use crate::repository::maven::artifact_repository::ArtifactRepository;

#[derive(Clone)]
pub struct VersionRepository {
    artifacts: ArtifactRepository,
}

impl VersionRepository {
    pub fn new(artifacts: ArtifactRepository) -> VersionRepository {
        return VersionRepository { artifacts };
    }

    pub async fn get_or_create_version(&self, pool: &Pool<MySql>, version: VersionDTO, access_token: AccessToken) -> Result<Option<Version>, Error> {
        let artifact = match self.artifacts.get_writable_artifact(pool, version.clone().to_artifact(), access_token).await? {
            Some(artifact) => artifact,
            None => return Ok(None),
        };
        let result = sqlx::query_as!(
            Version,
            "SELECT id, artifact_id, version FROM maven_version WHERE artifact_id = ? AND version = ?",
            artifact.id,
            version.version,
        ).fetch_optional(pool).await?;

        if let Some(version) = result {
            return Ok(Some(version));
        }

        let result = sqlx::query!("INSERT INTO maven_version (artifact_id, `version`) VALUES (?, ?);", artifact.id, version.version).execute(pool).await?;
        let version = sqlx::query_as!(Version, "SELECT id, artifact_id, version FROM maven_version WHERE id = ?", result.last_insert_id()).fetch_optional(pool).await?;
        return Ok(version);
    }
}
