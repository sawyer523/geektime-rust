use std::convert::Infallible;
use std::task::{Context, Poll};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Router,
};
use tower_http::services::ServeDir;
use tower_service::Service;
use tracing::info;

#[derive(Debug, Clone)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on {}", path, addr);

    let state = HttpServeState { path: path.clone() };
    let dir_service = ServeDir::new(path)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_zstd()
        .fallback(ListDirService {
            state: Arc::new(state.clone()),
        });

    let router = Router::new()
        .nest_service("/", dir_service)
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

async fn list_dir(
    State(state): State<Arc<HttpServeState>>,
    path: &str,
) -> Result<Response, StatusCode> {
    let full_path = std::path::Path::new(&state.path).join(path);
    if !full_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut html = String::from("<html><body><ul>");
    full_path.read_dir().unwrap().for_each(|entry| {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();
            let mut dash = String::from("");
            if entry.path().is_dir() {
                dash = String::from("/");
            }
            html.push_str(&format!(
                "<li><a href=\"{}{}\">{}{}</a></li>",
                name, dash, name, dash
            ));
        }
    });
    html.push_str("</ul></body></html>");
    let response = Html(html).into_response();
    Ok(response)
}

#[derive(Debug, Clone)]
struct ListDirService {
    state: Arc<HttpServeState>,
}

impl<B> Service<axum::http::Request<B>> for ListDirService
where
    B: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: axum::http::Request<B>) -> Self::Future {
        let state = self.state.clone();
        let path = req.uri().path()[1..].to_string();

        Box::pin(async move {
            match list_dir(State(state), &path).await {
                Ok(response) => Ok(response),
                Err(status_code) => {
                    let response = Response::builder()
                        .status(status_code)
                        .body("".into())
                        .unwrap();
                    Ok(response)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let path = PathBuf::from("./src");
        let state = Arc::new(HttpServeState { path: path.clone() });
        let result = list_dir(State(state), "process").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
