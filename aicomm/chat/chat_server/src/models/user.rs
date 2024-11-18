use std::mem;

use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use chat_core::{ChatUser, User};

use crate::{AppError, AppState};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[allow(unused)]
impl AppState {
    /// Find a user by email
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = \
            $1",
        )
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    // find user by id
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE id = \
            $1",
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /// verify email and password
    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = \
            $1",
        )
            .bind(&input.email)
            .fetch_optional(&self.pool)
            .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    // load ws_name, ws should exist
                    let ws = self.find_workspace_by_id(user.ws_id as _).await?.unwrap();
                    user.ws_name = ws.name;
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Create a new user
    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        let password_hash = hash_password(&input.password)?;

        let is_bot = input.email.ends_with("@bot.org");
        let mut user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (ws_id, email,fullname, password_hash, is_bot)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, ws_id, fullname, email, is_bot, created_at
            "#,
        )
            .bind(ws.id)
            .bind(&input.email)
            .bind(&input.fullname)
            .bind(password_hash)
            .bind(is_bot)
            .fetch_one(&self.pool)
            .await?;

        user.ws_name = ws.name.clone();

        if ws.owner_id == 0 {
            self.update_workspace_owner(ws.id as _, user.id as _)
                .await?;
        }
        Ok(user)
    }

    pub async fn fetch_chat_user_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)
            "#,
        )
            .bind(ids)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
impl CreateUser {
    pub fn new(ws: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            workspace: ws.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::AppState;

    use super::*;

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "Alice", "alice@acme.org", "password");
        let ret = state.create_user(&input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, input.email);
            }
            _ => panic!("Expecting EmailAlreadyExists error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateUser::new("acme", "cxn", "cxn@gmail.com", "password");
        let user = state.create_user(&input).await?;

        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&input.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);

        let input = SigninUser::new(&input.email, &input.password);
        let user = state.verify_user(&input).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        assert_eq!(verify_password(password, &password_hash)?, true);
        let is_valid = verify_password(password, &password_hash)?;
        assert!(is_valid);
        Ok(())
    }

    // find user by id should work
    #[tokio::test]
    async fn find_user_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?;
        assert!(user.is_some());
        Ok(())
    }
}
