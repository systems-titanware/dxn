//! Auth HTTP controller: login endpoint and Bearer guard.

use actix_web::{dev::ServiceRequest, web, HttpResponse, Responder};
use serde::Deserialize;

use super::sa_identity::SaIdentity;
use super::token;
use super::verify;

use crate::system::models::AppState;

/// Request body for POST /api/auth/login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response body for successful login: returns the Bearer token.
#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// POST /api/auth/login: validate username + password against SA file, return JWT Bearer token.
pub async fn login(
    app: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let Some(ref sa) = app.sa_identity else {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "SA not provisioned. Create .sa-identity.json to enable login."
        }));
    };
    if !verify::verify_username(&body.username, &sa.username) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid credentials." }));
    }
    match verify::verify_password(&body.password, sa) {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid credentials." }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": e }));
        }
    }
    match token::issue_token(&sa.id, &sa.jwt_signing_key) {
        Ok(t) => HttpResponse::Ok().json(LoginResponse { token: t }),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": e })),
    }
}

/// Extracts Bearer token from `Authorization: Bearer <token>` and verifies it using app state.
/// Returns (sa_id, None) on success, or (_, Some(response)) to return immediately.
pub fn check_bearer(
    req: &ServiceRequest,
    sa_identity: Option<&SaIdentity>,
) -> Result<String, HttpResponse> {
    let Some(ref sa) = sa_identity else {
        return Err(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "SA not provisioned. Authentication unavailable."
        })));
    };
    let auth_header = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let Some(header_value) = auth_header else {
        return Err(HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "Missing Authorization header." })));
    };
    let token = header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .map(str::trim);
    let Some(bearer_token) = token else {
        return Err(HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "Expected Authorization: Bearer <token>." })));
    };
    match token::verify_token(bearer_token, &sa.jwt_signing_key) {
        Ok(sa_id) => Ok(sa_id),
        Err(_) => Err(HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "Invalid or expired token." }))),
    }
}

/// Configures auth routes (login only; no guard).
pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/login/", web::post().to(login));
}
