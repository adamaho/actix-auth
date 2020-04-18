use actix_web::web;

use crate::db::DbPool;
use crate::models::key::{CheckKeyForm, Key};
use crate::utils::errors::ApiError;

///  Creates a user in the database
pub async fn check_key(
    pool: web::Data<DbPool>,
    web::Json(key_form): web::Json<CheckKeyForm>,
) -> Result<web::HttpResponse, ApiError> {
    let conn = pool.get()?;

    // check if the provided key is available
    let is_taken = web::block(move || Key::is_available(&key_form.key, &conn)).await?;

    if is_taken {
        Err(ApiError::InvalidBetaKey)
    } else {
        Ok(web::HttpResponse::Ok().finish())
    }
}
