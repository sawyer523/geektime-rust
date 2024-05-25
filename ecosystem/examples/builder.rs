use anyhow::Result;
use chrono::{Datelike, DateTime, Utc};
use derive_builder::Builder;

#[allow(unused)]
#[derive(Builder, Debug)]
#[builder(build_fn(name = "_priv_build"))]
struct User {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into, strip_option), default)]
    email: Option<String>,
    #[builder(setter(custom), default = "Utc::now()")]
    dob: DateTime<Utc>,
    #[builder(setter(skip))]
    age: u32,
    #[builder(default = "vec![]", setter(each(name = "skill", into)))]
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = User::builder()
        .name("Alice")
        .skill("Rust")
        .skill("Python")
        .dob("1990-01-01T00:00:00Z")
        .build()?;
    println!("{:?}", user);
    Ok(())
}

impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn build(&self) -> Result<User> {
        let mut user = self._priv_build()?;
        user.age = (Utc::now().year() - user.dob.year()) as _;
        Ok(user)
    }

    #[allow(unused)]
    pub fn dob(&mut self, value: &str) -> &mut Self {
        self.dob = DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();
        self
    }
}
