use axum::extract::{FromRequestParts, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use tracing::warn;

use crate::TokenVerifier;

pub async fn verify_token<T>(State(state): State<T>, req: Request, next: Next) -> Response
where
    T: TokenVerifier + Clone + Send + Sync + 'static,
{
    let (mut parts, body) = req.into_parts();
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                match state.verify(token) {
                    Ok(user) => {
                        let mut req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(user);
                        req
                    }
                    Err(e) => {
                        let msg = format!("verify token failed: {:?}", e);
                        warn!(msg);
                        return (StatusCode::FORBIDDEN, msg).into_response();
                    }
                }
            }
            Err(e) => {
                let msg = format!("parse Authorization header failed: {}", e);
                warn!(msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use axum::middleware::from_fn_with_state;
    use axum::response::IntoResponse;
    use axum::Router;
    use axum::routing::get;
    use tower::ServiceExt;

    use crate::{DecodingKey, EncodingKey, User};

    use super::*;

    #[derive(Clone)]
    struct AppState(Arc<AppStateInner>);

    struct AppStateInner {
        ek: EncodingKey,
        dk: DecodingKey,
    }

    impl TokenVerifier for AppState {
        type Error = jwt_simple::Error;

        fn verify(&self, token: &str) -> Result<User, Self::Error> {
            self.0.dk.verify(token)
        }
    }

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }
    #[tokio::test]
    async fn verify_token_middleware_should_work() -> Result<()> {
        let encoding_key =
            EncodingKey::load(include_str!("../../../chat_server/fixtures/encoding.pem"))?;
        let decoding_key =
            DecodingKey::load(include_str!("../../../chat_server/fixtures/decoding.pem"))?;
        let state = AppState(Arc::new(AppStateInner {
            ek: encoding_key,
            dk: decoding_key,
        }));
        let user = User::new(1, "cxn", "cxn@acme.org");
        let token = state.0.ek.sign(user)?;

        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .with_state(state);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // no token
        let req = Request::builder().uri("/").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // bad token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer bad-token")
            .body(Body::empty())?;
        let res = app.oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
