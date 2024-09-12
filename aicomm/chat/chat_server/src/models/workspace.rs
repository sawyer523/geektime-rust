use chat_core::{ChatUser, Workspace};

use crate::{AppError, AppState};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            "INSERT INTO workspaces (name, owner_id) VALUES ($1, $2) RETURNING id, name, owner_id, created_at",
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn fetch_chat_users(&self, id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1 order by id 
           "#,
        )
        .bind(id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn update_workspace_owner(
        &self,
        ws_id: u64,
        user_id: u64,
    ) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces 
            SET owner_id = $1 
            WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2 
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(user_id as i64)
        .bind(ws_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::models::CreateUser;

    use super::*;

    #[tokio::test]
    async fn create_workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;
        let input = CreateUser::new(&ws.name, "cxn", "cxn@acme.com", "password");
        let user = state.create_user(&input).await?;
        assert_eq!(&ws.name, "test");
        assert_eq!(user.ws_id, ws.id);

        let ws = state
            .update_workspace_owner(ws.id as _, user.id as _)
            .await?;
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let ws = state.find_workspace_by_name("acme").await?.unwrap();
        assert_eq!(ws.name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let users = state.fetch_chat_users(1).await?;
        assert_eq!(users.len(), 5);
        Ok(())
    }
}
