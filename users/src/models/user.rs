use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;
use common::errors::ApiError;

use crate::schema::users;

/// Database representation of a User
#[derive(Identifiable, Queryable, PartialEq, Associations, Serialize, Debug)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub key_id: uuid::Uuid,
    pub created_at: std::time::SystemTime,
}

#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "users"]
pub struct ViewableUser {
    pub id: i32,
    pub email: String,
}

impl User {
    pub fn find_all_users(conn: &PgConnection) -> Result<Vec<ViewableUser>, ApiError> {
        use crate::schema::users::dsl::users;
        use crate::schema::users::{email, id};

        let all_users = users.select((id, email)).load::<ViewableUser>(conn)?;

        Ok(all_users)
    }

    pub fn find_one(user_id: i32, conn: &PgConnection) -> Result<ViewableUser, ApiError> {
        use crate::schema::users::dsl::users;
        use crate::schema::users::{email, id};

        let user = users
            .find(user_id)
            .select((id, email))
            .first::<ViewableUser>(conn)?;

        Ok(user)
    }
}

/// Representation of a User Login model
#[derive(Validate, Debug, Deserialize)]
pub struct LoginUserForm {
    #[validate(email(code = "INVALID_EMAIL"))]
    pub email: String,
    pub password: String,
}

impl LoginUserForm {
    // Checks if the provided credentials are valid
    pub fn verify_user(self, conn: &PgConnection) -> Result<Option<User>, ApiError> {
        use crate::schema::users::dsl::*;
        use bcrypt::verify;

        let user = users
            .filter(email.eq(self.email))
            .first::<User>(conn)
            .optional()?;

        match user {
            Some(u) => {
                if !verify(self.password, &u.password).unwrap() {
                    return Ok(None);
                }

                return Ok(Some(u));
            }
            None => Ok(None),
        }
    }
}

/// Database representation of a User that can be inserted
#[derive(Insertable, Validate, Debug, Deserialize, Default)]
#[table_name = "users"]
pub struct NewUserForm {
    #[validate(email(code = "INVALID_EMAIL"))]
    pub email: String,
    pub password: String,
    pub key_id: uuid::Uuid,
}

impl NewUserForm {
    /// Creates a new user in the database
    pub fn create(mut self, conn: &PgConnection) -> Result<User, ApiError> {
        use crate::models::key::Key;
        use crate::schema::users::dsl::users as query_users;
        use bcrypt::hash;
        use diesel::insert_into;

        // validate the fields
        self.validate()?;

        // check if the key exists
        let has_key = Key::has_key(&self.key_id, conn)?;

        if has_key {
            // hashing the password
            self.password = hash(self.password, 4).unwrap();

            // insert new user in the database
            match insert_into(query_users)
                .values(&self)
                .returning(users::all_columns)
                .get_result(conn)
            {
                Ok(u) => return Ok(u),
                // if the same key is reused
                Err(_) => return Err(ApiError::InvalidBetaKey),
            }
        }

        // if the key doesnt match
        Err(ApiError::InvalidBetaKey)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::db::create_pool;

    #[test]
    fn it_gets_all_users() {
        let conn = create_pool().get().unwrap();

        let all_users = User::find_all_users(&conn);

        assert!(all_users.is_ok())
    }

    #[test]
    fn it_returns_err_for_invalid_email() {
        use crate::models::key::Key;
        use crate::schema::keys::dsl::*;

        let conn = create_pool().get().unwrap();

        let random_uuid = uuid::Uuid::new_v4();

        let new_key = Key { id: random_uuid };

        diesel::insert_into(keys)
            .values(&new_key)
            .execute(&conn)
            .expect("failed to insert key");

        let new_user = NewUserForm {
            email: "foo".to_string(),
            password: "password".to_string(),
            key_id: random_uuid,
        };

        let result = new_user.create(&conn);

        assert!(result.is_err());
    }

    #[test]
    fn it_creates_user() {
        use crate::models::key::Key;
        use crate::schema::keys::dsl::*;

        let conn = create_pool().get().unwrap();

        let random_uuid = uuid::Uuid::new_v4();

        let new_key = Key { id: random_uuid };

        diesel::insert_into(keys)
            .values(&new_key)
            .execute(&conn)
            .expect("failed to insert key");

        let new_user = NewUserForm {
            email: "foo2@bar.com".to_string(),
            password: "password".to_string(),
            key_id: random_uuid,
        };

        let result = new_user.create(&conn);

        assert!(result.is_ok());
    }

    #[test]
    fn it_verifies_user() {
        use crate::models::key::Key;
        use crate::schema::keys::dsl::*;

        let conn = create_pool().get().unwrap();

        let random_uuid = uuid::Uuid::new_v4();

        let new_key = Key { id: random_uuid };

        diesel::insert_into(keys)
            .values(&new_key)
            .execute(&conn)
            .expect("failed to insert key");

        let new_user = NewUserForm {
            email: "foo3@bar.com".to_string(),
            password: "password".to_string(),
            key_id: random_uuid,
        };

        new_user.create(&conn).expect("Failed to create new user");

        let login = LoginUserForm {
            email: "foo3@bar.com".to_string(),
            password: "password".to_string(),
        };

        let is_valid = login.verify_user(&conn);

        assert!(is_valid.is_ok());
    }
}
