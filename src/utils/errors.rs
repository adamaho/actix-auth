use actix_web::error::{BlockingError, JsonPayloadError};
use actix_web::{HttpResponse, ResponseError};
use diesel::{
    r2d2::PoolError,
    result::{DatabaseErrorKind, Error as DatabaseError},
};

use failure::Fail;
use serde::Serialize;
use validator::{ValidationErrors, ValidationErrorsKind};

/// Representation of an ApiError
#[derive(Fail, Debug)]
pub enum ApiError {
    #[fail(display = "An internal server error occured: {}", _0)]
    InternalServerError(String, String),
    #[fail(display = "A validation error occurred: {}", _0)]
    ValidationError(String, String, Vec<String>),
    #[fail(display = "Beta key is invalid or taken")]
    InvalidBetaKey,
    #[fail(display = "The provided email and password are invalid")]
    InvalidLogin,
    #[fail(display = "Unauthorized. Please login to continue")]
    Unauthorized,
}

/// Automatically convert ApiErrors to user facing errors
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::InternalServerError(code, message) => {
                HttpResponse::InternalServerError()
                    .json::<UserErrorResponse>((code, message).into())
            }
            ApiError::ValidationError(code, message, errors) => {
                HttpResponse::BadRequest().json::<UserErrorResponse>((code, message, errors).into())
            }
            ApiError::InvalidBetaKey => HttpResponse::BadRequest().json::<UserErrorResponse>(
                (
                    "INVALID_BETA_KEY",
                    "The provided beta key is taken or invalid",
                )
                    .into(),
            ),
            ApiError::InvalidLogin => HttpResponse::BadRequest().json::<UserErrorResponse>(
                (
                    "INVALID_LOGIN",
                    "The provided email and password are invalid",
                )
                    .into(),
            ),
            ApiError::Unauthorized => HttpResponse::Unauthorized().header("www-authenticate", "Bearer")
                .json::<UserErrorResponse>(("UNAUTHORIZED", "Please login to continue").into()),
        }
    }
}

impl From<JsonPayloadError> for ApiError {
    fn from(error: JsonPayloadError) -> ApiError {
        match error {
            JsonPayloadError::Deserialize(_) => ApiError::ValidationError(
                String::from("VALIDATION_ERROR"),
                String::from("A validation error occurred"),
                Vec::new(),
            ),
            _ => ApiError::ValidationError(
                String::from("VALIDATION_ERROR"),
                String::from("A validation error occurred"),
                Vec::new(),
            ),
        }
    }
}

/// Respresents the response a user will get when an ApiError occurs
#[derive(Serialize, Debug)]
struct UserErrorResponse {
    code: String,
    message: String,
    errors: Option<Vec<String>>,
}

/// Utility for converting a the strings to a usable UserErrorResponse
impl From<(&str, &str)> for UserErrorResponse {
    fn from(error: (&str, &str)) -> UserErrorResponse {
        UserErrorResponse {
            code: error.0.into(),
            message: error.1.into(),
            errors: None,
        }
    }
}

/// Utility for converting a the strings references to a usable UserErrorResponse
impl From<(&String, &String)> for UserErrorResponse {
    fn from(error: (&String, &String)) -> UserErrorResponse {
        UserErrorResponse {
            code: error.0.into(),
            message: error.1.into(),
            errors: None,
        }
    }
}

/// Utility for converting a tuple to a usable UserErrorResponse
impl From<(&String, &String, &Vec<String>)> for UserErrorResponse {
    fn from(error: (&String, &String, &Vec<String>)) -> UserErrorResponse {
        UserErrorResponse {
            code: error.0.into(),
            message: error.1.into(),
            errors: Some(error.2.to_vec().into()),
        }
    }
}

/// Converts a Database error to an ApiError
impl From<DatabaseError> for ApiError {
    fn from(error: DatabaseError) -> ApiError {
        match error {
            DatabaseError::DatabaseError(kind, _) => match kind {
                DatabaseErrorKind::UniqueViolation => ApiError::InternalServerError(
                    String::from("DATABASE_UNIQUNESS_ERROR"),
                    String::from("Uniquness database error occurred"),
                ),
                _ => ApiError::InternalServerError(
                    String::from("DATABASE_ERROR"),
                    String::from("Database error occurred"),
                ),
            },
            _ => ApiError::InternalServerError(
                String::from("UNKNOWN_DATABASE_ERROR"),
                String::from("Unknown database error occurred"),
            ),
        }
    }
}

/// Converts a web::block Blocking error to an ApiError
impl From<BlockingError<ApiError>> for ApiError {
    fn from(error: BlockingError<ApiError>) -> ApiError {
        match error {
            BlockingError::Error(api_error) => api_error,
            BlockingError::Canceled => ApiError::InternalServerError(
                String::from("BLOCKING_ERROR"),
                String::from("Thread execution was blocked"),
            ),
        }
    }
}

/// Converts a PoolError to an ApiError when a connection to the database pool cant be established
impl From<PoolError> for ApiError {
    fn from(_error: PoolError) -> ApiError {
        ApiError::InternalServerError(
            String::from("DATABASE_POOL_ERROR"),
            String::from("Could not get connection to database"),
        )
    }
}

/// Converts a validation error into an ApiError
impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> ApiError {
        let mut codes: Vec<String> = Vec::new();

        // build a vector of all error codes to return
        for (_, kind) in errors.into_errors().iter() {
            match kind {
                ValidationErrorsKind::Field(errors) => {
                    for e in errors {
                        codes.push(e.code.to_string());
                    }
                }
                _ => error!("Unhandled validation of kind: {:?}", kind),
            }
        }

        ApiError::ValidationError(
            String::from("VALIDATION_ERROR"),
            String::from("A validation error occurred"),
            codes,
        )
    }
}
