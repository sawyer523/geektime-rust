use std::collections::HashSet;
use std::sync::Arc;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{info, warn};

use chat_core::{Chat, Message};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    UpdateChatName(Chat),
    NewMessage(Message),
}

#[derive(Debug)]
struct Notification {
    name: String,
    // users being impacted, so we should send the notification to them
    user_ids: HashSet<u64>,
    event: Arc<AppEvent>,
}

// pg_notify('chat_updated', json_build_object('op', TG_OP, 'old', OLD, 'new', NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

// pg_notify('chat_message_created', row_to_json(NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    message: Message,
    members: Vec<i64>,
}

pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let mut listerner = PgListener::connect(&state.config.server.db_url).await?;
    listerner.listen("chat_updated").await?;
    listerner.listen("chat_message_created").await?;

    let mut stream = listerner.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(notify)) = stream.next().await {
            // TODO: parse notification and sent to users
            info!("notification: {:?}", notify);
            let nofitication = Notification::load(notify.channel().as_ref(), notify.payload())?;
            let users = &state.users;
            for user_id in nofitication.user_ids {
                if let Some(sender) = users.get(&user_id) {
                    if let Err(e) = sender.send(nofitication.event.clone()) {
                        warn!("send notification to user {} failed: {:?}", user_id, e);
                    }
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

impl Notification {
    fn load(r#type: &str, payload: &str) -> anyhow::Result<Self> {
        match r#type {
            "chat_updated" => {
                let payload: ChatUpdated = serde_json::from_str(payload)?;
                let mut ids =
                    get_affected_chat_user_ids(payload.old.as_ref(), payload.new.as_ref());
                let name = get_affected_chat_name(payload.old.as_ref(), payload.new.as_ref());
                let event = match payload.op.as_str() {
                    "INSERT" => AppEvent::NewChat(payload.new.expect("new should exist")),
                    "UPDATE" => {
                        if !name.is_empty() {
                            let chat = payload.new.expect("new should exist");
                            ids = chat.members.iter().map(|v| *v as u64).collect();
                            AppEvent::UpdateChatName(chat)
                        } else {
                            AppEvent::AddToChat(payload.new.expect("new should exist"))
                        }
                    }
                    "DELETE" => AppEvent::RemoveFromChat(payload.old.expect("old should exist")),
                    _ => return Err(anyhow::anyhow!("invalid op")),
                };
                Ok(Self {
                    name,
                    user_ids: ids,
                    event: Arc::new(event),
                })
            }
            "chat_message_created" => {
                let payload: ChatMessageCreated = serde_json::from_str(payload)?;
                let user_ids = payload.members.iter().map(|v| *v as u64).collect();
                Ok(Self {
                    name: "".to_string(),
                    user_ids,
                    event: Arc::new(AppEvent::NewMessage(payload.message)),
                })
            }
            _ => Err(anyhow::anyhow!("invalid type")),
        }
    }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<u64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            // diff old and new members, if identical, no need to notify, otherwise notify the diff
            let old_members: HashSet<_> = old.members.iter().map(|v| *v as u64).collect();
            let new_members: HashSet<_> = new.members.iter().map(|v| *v as u64).collect();
            if old_members == new_members {
                HashSet::new()
            } else {
                old_members.union(&new_members).copied().collect()
            }
        }
        (Some(old), None) => old.members.iter().map(|v| *v as u64).collect(),
        (None, Some(new)) => new.members.iter().map(|v| *v as u64).collect(),
        _ => HashSet::new(),
    }
}

// if name changed, return the new name, otherwise return empty string
fn get_affected_chat_name(old: Option<&Chat>, new: Option<&Chat>) -> String {
    match (old, new) {
        (Some(old), Some(new)) => {
            if old.name == new.name {
                "".to_string()
            } else {
                match &new.name {
                    Some(name) => name.to_string(),
                    None => "".to_string(),
                }
            }
        }
        _ => "".to_string(),
    }
}

// fn get_affected_chat_name(old: Option<&Chat>, new: Option<&Chat>) -> &str {
//     match (old, new) {
//         (Some(old), Some(new)) => {
//             if old.name == new.name {
//                 ""
//             } else {
//                 if let Some(name) = match &new.name {
//                     Some(name) => name.as_str(),
//                     None => "",
//                 };
//             }
//         }
//         _ => "",
//     }
// }
