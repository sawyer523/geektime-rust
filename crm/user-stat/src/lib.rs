use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use sqlx::PgPool;
use tonic::{async_trait, Request, Response, Status};

pub use config::AppConfig;

use crate::pb::{QueryRequest, RawQueryRequest, User};
use crate::pb::user_stats_server::{UserStats, UserStatsServer};

pub mod abi;
mod config;
pub mod pb;

#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;
#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    type RawQueryStream = ResponseStream;
    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let req = request.into_inner();
        self.raw_query(req).await
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to database");
        let inner = UserStatsServiceInner { config, pool };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
