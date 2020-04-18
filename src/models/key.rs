use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::schema::keys;
use crate::utils::errors::ApiError;

/// Database representation of a Beta Key
#[derive(Identifiable, Insertable, Queryable, Serialize, Debug)]
#[table_name = "keys"]
pub struct Key {
    pub id: uuid::Uuid,
}

impl Key {
    /// Checks if the provided key is vaild
    pub fn has_key(key: &uuid::Uuid, conn: &PgConnection) -> Result<bool, ApiError> {
        use crate::schema::keys::dsl::*;

        let beta_key = keys.find(key).first::<Self>(conn).optional()?;

        if beta_key.is_some() {
            return Ok(true);
        }

        return Ok(false);
    }

    /// Checks if the key is both valid and available
    pub fn is_available(key: &uuid::Uuid, conn: &PgConnection) -> Result<bool, ApiError> {
        use crate::schema::keys::dsl::*;
        use crate::schema::users::dsl::*;

        let is_taken = users
            .filter(key_id.eq(key))
            .first::<User>(conn)
            .optional()?;

        let is_valid = keys.find(key).first::<Self>(conn).optional()?;

        if is_taken.is_some() & is_valid.is_some() {
            return Ok(true);
        }

        return Ok(false);
    }
}

/// Check Key form  used to check if a key is valid
#[derive(Deserialize, Debug)]
pub struct CheckKeyForm {
    pub key: uuid::Uuid,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::db::create_pool;

    #[test]
    fn it_returns_false_for_missing_key() {
        let conn = create_pool().get().unwrap();

        let random_uuid = uuid::Uuid::new_v4();

        let has_key = Key::has_key(&random_uuid, &conn).expect("failed to get key");

        assert!(!has_key)
    }

    #[test]
    fn it_returns_true_for_existing_key() {
        use crate::schema::keys::dsl::*;

        let conn = create_pool().get().unwrap();

        let random_uuid = uuid::Uuid::new_v4();

        let new_key = Key { id: random_uuid };

        diesel::insert_into(keys)
            .values(&new_key)
            .execute(&conn)
            .expect("failed to insert key");

        let has_key = Key::has_key(&random_uuid, &conn).expect("failed to get key");

        assert!(has_key);
    }

    #[test]
    fn it_returns_false_for_valid_but_taken_key() {
        use crate::models::user::NewUserForm;
        use crate::schema::keys::dsl::*;

        let conn = create_pool().get().unwrap();

        let random_uuid = uuid::Uuid::new_v4();

        let new_key = Key { id: random_uuid };

        diesel::insert_into(keys)
            .values(&new_key)
            .execute(&conn)
            .expect("failed to insert key");

        let new_user = NewUserForm {
            email: "foo1@bar.com".to_string(),
            password: "password".to_string(),
            key_id: random_uuid,
        };

        new_user.create(&conn).expect("failed to create user");

        let is_available = Key::is_available(&random_uuid, &conn).unwrap();

        assert!(is_available);
    }
}
