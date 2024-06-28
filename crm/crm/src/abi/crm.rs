use crate::pb::User;
use prost_types::Timestamp;

impl User {
    pub fn new(id: u64, name: &str, email: &str) -> Self {
        User {
            id,
            name: name.to_string(),
            email: email.to_string(),
            created_at: Some(Timestamp::default()),
        }
    }
}
