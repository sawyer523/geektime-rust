use anyhow::Result;
use tonic::{async_trait, Request, Response, Status, transport::Channel};
use tonic::codegen::InterceptedService;

pub use abi::*;
pub use config::*;
use crm_metadata::pb::metadata_client::MetadataClient;
use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use send::pb::notification_client::NotificationClient;
use user_stat::pb::user_stats_client::UserStatsClient;

mod abi;
mod config;
pub mod pb;
pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        self.welcome(request.into_inner()).await
    }

    async fn recall(
        &self,
        request: Request<RecallRequest>,
    ) -> Result<Response<RecallResponse>, Status> {
        self.recall(request.into_inner()).await
    }

    async fn remind(
        &self,
        request: Request<RemindRequest>,
    ) -> Result<Response<RemindResponse>, Status> {
        self.remind(request.into_inner()).await
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;
        Ok(Self {
            config,
            user_stats,
            notification,
            metadata,
        })
    }

    pub fn into_server(self) -> Result<InterceptedService<CrmServer<CrmService>, DecodingKey>> {
        let dk = DecodingKey::load(&self.config.auth.pk)?;
        Ok(CrmServer::with_interceptor(self, dk))
    }
}
