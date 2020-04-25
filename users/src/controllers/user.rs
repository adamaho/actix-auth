use actix_web::web;
use common::errors::ApiError;

use crate::db::DbPool;
use crate::models::user::{LoginUserForm, NewUserForm, User};
use crate::utils::token::Token;

///  Returns all users
pub async fn get(pool: web::Data<DbPool>) -> Result<web::HttpResponse, ApiError> {
    let conn = pool.get()?;
    info!("asdfasdfasdf");
    // get all users
    let users = web::block(move || User::find_all_users(&conn)).await?;

    // respond with all users
    Ok(web::HttpResponse::Ok().json(users))
}

///  Creates a user in the database
pub async fn create(
    pool: web::Data<DbPool>,
    web::Json(new_user): web::Json<NewUserForm>,
) -> Result<web::HttpResponse, ApiError> {
    let conn = pool.get()?;

    // create user in database
    let user = web::block(move || new_user.create(&conn)).await?;

    // create token for the user
    let token = Token::from_user(&user).encode();

    // respond with the token instead of the user
    Ok(web::HttpResponse::Ok().json(token))
}

/// Creates a jwt token for the user to use for requests
pub async fn login(
    pool: web::Data<DbPool>,
    web::Json(creds): web::Json<LoginUserForm>,
) -> Result<web::HttpResponse, ApiError> {
    let conn = pool.get()?;

    // Verifies the users login information
    let valid_user = web::block(move || creds.verify_user(&conn)).await?;

    match valid_user {
        Some(u) => {
            let token = Token::from_user(&u).encode();

            return Ok(web::HttpResponse::Ok().json(token));
        }
        None => Err(ApiError::InvalidLogin),
    }
}
