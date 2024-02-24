use actix_web::HttpResponse;

use crate::model::AccessToken;

pub mod authenticator;

pub enum AuthenticationResult {
    Success(AccessToken),
    Unauthorized,
    Forbidden,
    Error(String),
}

impl AuthenticationResult {
    pub fn as_response(&self) -> HttpResponse {
        return match self {
            AuthenticationResult::Success(_) => HttpResponse::Ok().into(),
            AuthenticationResult::Unauthorized => HttpResponse::Unauthorized().body("Unauthorized"),
            AuthenticationResult::Forbidden => HttpResponse::Forbidden().body("Forbidden"),
            AuthenticationResult::Error(reason) => HttpResponse::InternalServerError().body(reason.to_string()),
        };
    }

    pub fn is_success(&self) -> bool {
        return matches!(*self, AuthenticationResult::Success(_));
    }
}
