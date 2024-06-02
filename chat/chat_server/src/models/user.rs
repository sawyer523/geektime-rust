use crate::User;

impl User {
    pub async fn find_by_email(
        email: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Option<User>, thiserror::Error> {
        let user = sqlx::query_as!(User, r#"SELECT * FROM users WHERE email = $1"#, email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }
}
