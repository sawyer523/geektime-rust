use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use utoipa::ToSchema;

use chat_core::{Chat, ChatType};

use crate::{AppError, AppState};

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PatchChat {
    pub name: Option<String>,
    pub members: Option<Vec<i64>>,
    pub public: Option<bool>,
}

#[allow(unused)]
impl AppState {
    pub async fn create_chat(&self, input: CreateChat, ws_id: u64) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "members must be more than 1".to_string(),
            ));
        }

        if 8 < len && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "name is required for group chat with more than 8 members".to_string(),
            ));
        }

        let users = self.fetch_chat_user_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "some members do not exist".to_string(),
            ));
        };

        let chat_type = generate_chat_type(&input.name, len, input.public);

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_chats(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn get_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete_chat(&self, id: u64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM chats WHERE id = $1")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_chat(
        &self,
        input: &PatchChat,
        id: u64,
        ws_id: i64,
    ) -> Result<Chat, AppError> {
        let pool = &self.pool;
        let mut chat = match self.get_chat_by_id(id).await? {
            Some(chat) => chat,
            None => return Err(AppError::UpdateChatError("chat not found".to_string())),
        };

        if chat.ws_id != ws_id {
            return Err(AppError::PermissionDenied(
                "chat is not belong to user".to_string(),
            ));
        }

        let mut updated = vec![];
        match &input.members {
            Some(members) => {
                let len = members.len();
                if len < 2 {
                    return Err(AppError::UpdateChatError(
                        "members must be more than 1".to_string(),
                    ));
                }

                if 8 < len && input.name.is_none() {
                    return Err(AppError::UpdateChatError(
                        "name is required for group chat with more than 8 members".to_string(),
                    ));
                }

                let users = self.fetch_chat_user_by_ids(&members).await?;
                if users.len() != len {
                    return Err(AppError::UpdateChatError(
                        "some members do not exist".to_string(),
                    ));
                };

                if members != &chat.members {
                    chat.members = members.to_vec();
                    updated.push("members");
                }
            }
            _ => {}
        }

        match (&input.name, &chat.name) {
            (Some(name), Some(chat_name)) => {
                if name != chat_name {
                    chat.name = Some(name.to_string());
                    updated.push("name");
                }
            }
            (Some(name), None) => {
                chat.name = Some(name.to_string());
                updated.push("name");
            }
            _ => {}
        }

        let mut public = false;
        match input.public {
            Some(pubs) => {
                public = pubs;
            }
            _ => {}
        }

        let chat_type = generate_chat_type(&chat.name, chat.members.len(), public);
        if chat_type != chat.r#type {
            chat.r#type = chat_type;
            updated.push("type");
        }

        if updated.is_empty() {
            return Ok(chat);
        }

        // update chat
        // new query
        let mut query = QueryBuilder::new("UPDATE chats SET ");
        for (i, column) in updated.iter().enumerate() {
            if 0 < i {
                query.push(", ");
            }
            match column.as_ref() {
                "members" => {
                    query.push(column).push(" = ").push_bind(&chat.members);
                }
                "name" => {
                    query.push(column).push(" = ").push_bind(&chat.name);
                }
                "type" => {
                    query.push(column).push(" = ").push_bind(&chat.r#type);
                }
                _ => {}
            }
        }

        query.push(" WHERE id = ").push_bind(id as i64);
        query.push(" RETURNING id, ws_id, name, type, members, created_at");
        let chat = query.build_query_as().fetch_one(pool).await?;

        Ok(chat)
    }

    pub(crate) async fn is_chat_member(
        &self,
        chat_id: u64,
        user_id: u64,
    ) -> Result<bool, AppError> {
        let is_memeber = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(is_memeber.is_some())
    }
}

fn generate_chat_type(name: &Option<String>, len: usize, public: bool) -> ChatType {
    let chat_type = match (name, len) {
        (None, 2) => ChatType::Single,
        (None, _) => ChatType::Group,
        (Some(_), _) => {
            if public {
                ChatType::PublicChannel
            } else {
                ChatType::PrivateChannel
            }
        }
    };
    chat_type
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };

        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
impl PatchChat {
    pub fn new(name: &str, members: &[i64], public: Option<bool>) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };

        let members = if members.is_empty() {
            None
        } else {
            Some(members.to_vec())
        };

        Self {
            name,
            members,
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state.create_chat(input, 1).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);

        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("general-test-2", &[1, 2, 3], true);
        let chat = state.create_chat(input, 1).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state
            .get_chat_by_id(1)
            .await
            .expect("get chat by id failed")
            .unwrap();

        assert_eq!(chat.id, 1);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.name.unwrap(), "general");

        Ok(())
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_chats(1).await?;
        assert_eq!(chats.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn chat_update_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateChat::new("general5", &[1, 2, 3, 4, 5], false);
        let chat = state.create_chat(input, 1).await?;

        let input = PatchChat::new("general7", &[1, 2, 4, 5], Some(true));
        let chat2 = state.update_chat(&input, chat.id as _, 2).await;
        assert!(chat2.is_err());

        let chat = state.update_chat(&input, chat.id as _, 1).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 4);
        assert_eq!(chat.name.unwrap(), "general7");
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        Ok(())
    }

    #[tokio::test]
    async fn chat_delete_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        state.delete_chat(2).await?;
        let chat = state.get_chat_by_id(2).await?;
        assert!(chat.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn chat_is_member_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(1, 1).await.expect("is member failed");
        assert!(is_member);

        // user 6 doesn't exist
        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);

        // chat 10 doesn't exist
        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);

        // user 4 is not a member of chat 2
        let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
        assert!(!is_member);

        Ok(())
    }
}
