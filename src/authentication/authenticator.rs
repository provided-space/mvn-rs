use actix_web_httpauth::headers::authorization::Basic;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hmac::digest::{KeyInit};
use hmac::Hmac;
use jwt::VerifyWithKey;
use serde::Deserialize;
use sha2::Sha256;
use sqlx::{MySql, Pool};

use crate::authentication::AuthenticationResult;
use crate::authentication::AuthenticationResult::{Error, Forbidden, Success, Unauthorized};
use crate::model::AccessToken;

#[derive(Clone)]
pub struct Authenticator {
    secret: String,
}

impl Authenticator {
    pub fn new(secret: String) -> Authenticator {
        return Authenticator { secret };
    }

    pub async fn authenticate(&self, authentication: &Basic, pool: &Pool<MySql>) -> AuthenticationResult {
        let password = match authentication.password() {
            Some(password) => password,
            None => return Unauthorized,
        };

        let key: Hmac<Sha256> = match Hmac::new_from_slice(self.secret.as_bytes()) {
            Ok(key) => key,
            Err(_) => return Error("Invalid authentication key length".to_owned()),
        };

        let claims: IdentityClaim = match password.verify_with_key(&key) {
            Ok(claim) => claim,
            Err(_) => return Unauthorized,
        };

        let access_token = match self.find_access_token(pool, claims.id, authentication.user_id()).await {
            Ok(access_token) => access_token,
            Err(result) => return result,
        };

        return match PasswordHash::new(&access_token.credentials) {
            Ok(hash) => match Argon2::default().verify_password(claims.credentials.as_bytes(), &hash) {
                Ok(_) => Success(access_token),
                Err(_) => Forbidden,
            },
            Err(_) => Forbidden,
        };
    }

    async fn find_access_token(&self, pool: &Pool<MySql>, id: u32, user_id: &str) -> Result<AccessToken, AuthenticationResult> {
        let query = sqlx::query_as!(
            AccessToken,
            "SELECT access_token.id, access_token.user_id, access_token.credentials
            FROM access_token
            INNER JOIN user ON access_token.user_id = user.id
            WHERE access_token.id = ? AND user.name = ?",
            id,
            user_id,
        );

        return match query.fetch_optional(pool).await {
            Ok(Some(access_token)) => Ok(access_token),
            Ok(None) => return Err(Forbidden),
            Err(_) => return Err(Error("Querying failed.".to_string())),
        };
    }
}

#[derive(Deserialize)]
struct IdentityClaim {
    id: u32,
    credentials: String,
}
