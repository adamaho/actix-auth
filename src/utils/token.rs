use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::utils::errors::ApiError;
use crate::models::user::User;

/// Represents the contents of a jwt
#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub sub: i32,
    pub email: String,
    pub company: String,
    pub iat: u64,
    pub exp: u64,
}

impl Token {
    /// Creates an instance of a token from the provided user
    pub fn from_user(user: &User) -> Self {
        let start = SystemTime::now();
        let iat = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let exp = iat + Duration::from_secs(60 * 60 * 24 * 7).as_secs();

        Token {
            sub: user.id.clone(),
            email: user.email.clone(),
            company: "tallii".to_string(),
            iat,
            exp,
        }
    }

    /// Decodes the provided token to the Token struct
    pub fn decode(token: &str) -> Result<jsonwebtoken::TokenData<Token>, ApiError> {
        let secret = env::var("USERS_SECRET").expect("secret has not been defined");
        let token = decode::<Token>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        );

        match token {
            Ok(c) => {
                return Ok(c);
            }
            Err(_) => {
                return Err(ApiError::Unauthorized);
            }
        }
    }

    /// Encodes the provided token struct to a string
    pub fn encode(&self) -> String {
        let secret = env::var("USERS_SECRET").expect("secret has not been defined");
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::models::user::User;

    pub fn create_token() -> Token {
        let user = User {
            id: 1,
            email: "foo@bar.com".to_string(),
            password: "password".to_string(),
            key_id: uuid::Uuid::new_v4(),
            created_at: std::time::SystemTime::now(),
        };

        Token::from_user(&user)
    }

    #[test]
    pub fn it_creates_token_from_user() {
        let token = create_token();

        assert!(token.email == "foo@bar.com".to_string());
    }

    #[test]
    pub fn it_decodes_token() {
        let token = create_token();
        let encoded_token = token.encode();

        let decoded_token = Token::decode(&encoded_token).expect("Failed to decode token");

        assert!(decoded_token.claims.email == token.email);
    }
}
