use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use config::AppConfig;
use pb::{
    Content,
    MaterializeRequest, metadata_server::{Metadata, MetadataServer},
};

pub mod pb;

mod abi;
mod config;

#[derive(Clone)]
pub struct MetadataService {
    inner: Arc<MetadataServiceInner>,
}

#[allow(unused)]
pub struct MetadataServiceInner {
    config: AppConfig,
}
type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;

#[async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;

    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> Result<Response<Self::MaterializeStream>, Status> {
        let req = request.into_inner();
        self.materialize(req).await
    }
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        let inner = MetadataServiceInner { config };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}

impl Deref for MetadataService {
    type Target = MetadataServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
