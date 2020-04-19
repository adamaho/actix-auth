use actix_web::web;
use actix_web::Error;

use crate::controllers::{key, user};
use crate::utils::errors::ApiError;
use crate::utils::token::Token;

use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;

/// Middleware validator used to ensure the provided bearer token is valid
async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    let token = credentials.token();

    match Token::decode(&token) {
        Ok(_) => Ok(req),
        Err(_) => Err(ApiError::Unauthorized.into()),
    }
}

/// Defines all of the routes for the application
pub fn define_routes(cfg: &mut web::ServiceConfig) {
    let middleware = HttpAuthentication::bearer(validator);
    cfg.service(
        web::resource("/users")
            // .wrap(middleware)
            .route(web::get().to(user::get)),
    )
    // public routes
    .service(web::resource("/keys").route(web::post().to(key::check_key)))
    .service(web::resource("/signup").route(web::post().to(user::create)))
    .service(web::resource("/login").route(web::post().to(user::login)));
}
