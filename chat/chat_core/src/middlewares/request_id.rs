use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use crate::middlewares::REQUEST_ID_HEADER;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if x-request-id exist, do nothing, otherwise generate a new noe
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let request_id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&request_id) {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;

    let Some(v) = id else {
        return res;
    };
    res.headers_mut().insert(REQUEST_ID_HEADER, v);
    res
}
