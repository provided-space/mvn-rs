pub mod maven;

use sqlx::FromRow;

#[derive(FromRow)]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[derive(FromRow)]
pub struct AccessToken {
    pub id: u32,
    pub user_id: u32,
}
