use sqlx::{Error, MySql, Pool};

use crate::model::maven::{Artifact, Version};
use crate::coordinates::Version as VersionDTO;
use crate::coordinates::Artifact as ArtifactDTO;
use crate::model::AccessToken;

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

#[derive(Clone)]
pub struct ArtifactRepository {}

impl ArtifactRepository {
    pub fn new() -> ArtifactRepository {
        return ArtifactRepository {};
    }

    pub async fn get_writable_artifact(&self, pool: &Pool<MySql>, artifact: ArtifactDTO, access_token: AccessToken) -> Result<Option<Artifact>, Error> {
        return sqlx::query_as!(
            Artifact,
            "SELECT artifact.id, artifact.group_id, artifact.name, artifact.public as `public: bool`
            FROM maven_artifact artifact
            INNER JOIN maven_group g ON artifact.group_id = g.id
            INNER JOIN maven_permission permission ON permission.artifact_id = artifact.id AND permission.access_token_id = ?
            WHERE g.name = ? AND artifact.name = ? AND permission.write",
            access_token.id,
            artifact.group,
            artifact.artifact,
        ).fetch_optional(pool).await;
    }
}
