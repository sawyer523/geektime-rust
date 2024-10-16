use std::fmt;

use axum::middleware::from_fn;
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

pub use auth::verify_token;
use request_id::set_request_id;
use service_time::ServiceTimeLayer;

use crate::User;

mod auth;
mod request_id;
mod service_time;

pub trait TokenVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}

const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVICE_TIME_HEADER: &str = "x-service-time";

pub fn set_layer(app: Router) -> Router {
    app.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
    )
        .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
        .layer(from_fn(set_request_id))
        .layer(ServiceTimeLayer)
}
