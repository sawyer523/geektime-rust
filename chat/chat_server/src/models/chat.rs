use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder};

use crate::AppError;
use crate::models::{Chat, ChatType, ChatUser};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatchChat {
    pub name: Option<String>,
    pub members: Option<Vec<i64>>,
    pub public: Option<bool>,
}

impl Chat {
    pub async fn create(input: CreateChat, ws_id: u64, pool: &PgPool) -> Result<Chat, AppError> {
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

        let users = ChatUser::fetch_by_ids(&input.members, pool).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "some members do not exist".to_string(),
            ));
        };

        let chat_type = Self::generate_type(&input.name, len, input.public);

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
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_all(ws_id: u64, pool: &PgPool) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    pub async fn get_by_id(id: u64, pool: &PgPool) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete(id: u64, pool: &PgPool) -> Result<(), AppError> {
        sqlx::query("DELETE FROM chats WHERE id = $1")
            .bind(id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        input: &PatchChat,
        id: u64,
        ws_id: i64,
        pool: &PgPool,
    ) -> Result<Chat, AppError> {
        let mut chat = match Self::get_by_id(id, pool).await? {
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

                let users = ChatUser::fetch_by_ids(&members, pool).await?;
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

        let chat_type = Self::generate_type(&chat.name, chat.members.len(), public);
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

    fn generate_type(name: &Option<String>, len: usize, public: bool) -> ChatType {
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

    use crate::test_util::get_test_pool;

    use super::*;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = Chat::create(input, 1, &pool).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);

        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(input, 1, &pool).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let chat = Chat::get_by_id(1, &pool)
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
        let (_tdb, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all(1, &pool).await?;
        assert_eq!(chats.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn chat_update_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = CreateChat::new("general5", &[1, 2, 3, 4, 5], false);
        let chat = Chat::create(input, 1, &pool).await?;

        let input = PatchChat::new("general7", &[1, 2, 4, 5], Some(true));
        let chat2 = Chat::update(&input, chat.id as _, 2, &pool).await;
        assert!(chat2.is_err());

        let chat = Chat::update(&input, chat.id as _, 1, &pool).await?;

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 4);
        assert_eq!(chat.name.unwrap(), "general7");
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        Ok(())
    }

    #[tokio::test]
    async fn chat_delete_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        Chat::delete(1, &pool).await?;
        let chat = Chat::get_by_id(1, &pool).await?;
        assert!(chat.is_none());
        Ok(())
    }
}
