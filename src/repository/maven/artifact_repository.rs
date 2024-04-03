use sqlx::{Error, MySql, Pool};
use crate::model::AccessToken;
use crate::model::maven::{Artifact, MavenPermission};

#[derive(Clone)]
pub struct ArtifactRepository {}

impl ArtifactRepository {
    pub fn new() -> ArtifactRepository {
        return ArtifactRepository {};
    }

    pub async fn get_artifact(&self, pool: &Pool<MySql>, artifact: crate::coordinates::Artifact) -> Result<Option<Artifact>, Error> {
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

    pub async fn can_read(&self, pool: &Pool<MySql>, artifact: Artifact, access_token: AccessToken) -> Result<bool, Error> {
        return sqlx::query_as!(
            MavenPermission,
            "SELECT permission.id, permission.access_token_id, permission.entity_type, permission.artifact_id, permission.read as `read: bool`, permission.write as `write: bool`
            FROM maven_permission permission
            WHERE permission.artifact_id = ? AND permission.access_token_id = ?",
            artifact.id,
            access_token.id,
        )
            .fetch_optional(pool)
            .await
            .map(|result| return match result {
                Some(permission) => permission.read,
                None => false,
            });
    }

    pub async fn get_writable_artifact(&self, pool: &Pool<MySql>, artifact: crate::coordinates::Artifact, access_token: AccessToken) -> Result<Option<Artifact>, Error> {
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