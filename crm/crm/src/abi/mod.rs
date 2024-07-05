use std::sync::Arc;

use chrono::{Duration, Utc};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::warn;

pub use auth::{DecodingKey, User};
use crm_metadata::pb::{Content, MaterializeRequest};
use send::pb::SendRequest;
use user_stat::pb::QueryRequest;

use crate::CrmService;
use crate::pb::{
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};

mod auth;

impl CrmService {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<Response<WelcomeResponse>, Status> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.content_ids))
            .await?
            .into_inner();

        let contents: Vec<Content> = contents
            .filter_map(|v| async move { v.ok() })
            .collect()
            .await;
        let contents = Arc::new(contents);

        let (tx, rx) = mpsc::channel(1024);

        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Welcome".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });
        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        let ret = WelcomeResponse { id: request_id };
        Ok(Response::new(ret))
    }

    pub async fn recall(&self, req: RecallRequest) -> Result<Response<RecallResponse>, Status> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.last_visit_interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("last_visited_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.content_ids))
            .await?
            .into_inner();

        let contents: Vec<Content> = contents
            .filter_map(|v| async move { v.ok() })
            .collect()
            .await;
        let contents = Arc::new(contents);

        let (tx, rx) = mpsc::channel(1024);

        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Recall".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });
        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        let ret = RecallResponse { id: request_id };
        Ok(Response::new(ret))
    }

    pub async fn remind(&self, req: RemindRequest) -> Result<Response<RemindResponse>, Status> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.last_visit_interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("last_watched_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let (tx, rx) = mpsc::channel(1024);

        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Remind".to_string(), sender, &[user.email], &[]);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });
        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        let ret = RemindResponse { id: request_id };
        Ok(Response::new(ret))
    }
}
