use std::{pin::Pin, sync::Arc};

use futures::Stream;
use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use config::AppConfig;
use pb::{notification_server::Notification, send_request::Msg, SendRequest, SendResponse};

pub mod pb;

mod abi;
mod config;

#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}

#[allow(unused)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[async_trait]
impl Notification for NotificationService {
    type sendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> Result<Response<Self::sendStream>, Status> {
        let stream = request.into_inner();
        self.send(stream).await
    }
}
