use sqlx::{Error, MySql, Pool};

use crate::model::maven::{Artifact, Version};
use crate::coordinates::Version as VersionDTO;
use crate::coordinates::Artifact as ArtifactDTO;

#[derive(Clone)]
pub struct VersionRepository {
    artifacts: ArtifactRepository,
}

impl VersionRepository {
    pub fn new(artifacts: ArtifactRepository) -> VersionRepository {
        return VersionRepository { artifacts };
    }

    pub async fn get_or_create_version(&self, pool: &Pool<MySql>, version: VersionDTO) -> Result<Option<Version>, Error> {
        let artifact = match self.artifacts.get_artifact(pool, version.clone().to_artifact()).await? {
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

    pub async fn get_artifact(&self, pool: &Pool<MySql>, artifact: ArtifactDTO) -> Result<Option<Artifact>, Error> {
        return sqlx::query_as!(
            Artifact,
            "SELECT artifact.id, artifact.group_id, artifact.name, artifact.public as `public: bool`
            FROM maven_artifact artifact
            INNER JOIN maven_group g ON artifact.group_id = g.id
            WHERE g.name = ? AND artifact.name = ?",
            artifact.group,
            artifact.artifact,
        ).fetch_optional(pool).await;
    }
}
