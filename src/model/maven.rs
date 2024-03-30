use sqlx::FromRow;

#[derive(FromRow)]
pub struct Group {
    pub id: u32,
    pub name: u32,
}

#[derive(FromRow)]
pub struct Artifact {
    pub id: u32,
    pub group_id: u32,
    pub name: String,
    pub public: bool,
}

#[derive(FromRow)]
pub struct Version {
    pub id: u32,
    pub artifact_id: u32,
    pub version: String,
}

#[derive(FromRow)]
pub struct File {
    pub id: u64,
    pub version_id: Option<u32>,
    pub name: String,
    pub uri: String,
    pub path: String,
}

#[derive(FromRow)]
pub struct MavenPermission {
    pub id: u32,
    pub access_token_id: u32,
    pub entity_type: String,
    pub artifact_id: Option<u32>,
    pub read: bool,
    pub write: bool,
}
