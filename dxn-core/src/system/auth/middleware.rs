//! Middleware that requires a valid Authorization: Bearer token for protected routes.

use actix_web::body::BoxBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpMessage;

use crate::system::models::AppState;

use super::controller::check_bearer;

/// Extension type for the authenticated SA id (inserted by the middleware).
#[derive(Clone, Debug)]
pub struct AuthenticatedSaId(pub String);

/// Middleware that checks Authorization: Bearer and returns 401/503 on failure.
/// Use with actix_web::middleware::from_fn(sa_auth_middleware).
pub async fn sa_auth_middleware(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let app = req.app_data::<web::Data<AppState>>();
    let sa_identity = app.and_then(|a| a.sa_identity.as_ref());

    match check_bearer(&req, sa_identity) {
        Ok(sa_id) => {
            req.request().extensions_mut().insert(AuthenticatedSaId(sa_id));
            next.call(req).await
        }
        Err(resp) => {
            let (request, _) = req.into_parts();
            Ok(ServiceResponse::new(request, resp))
        }
    }
}
