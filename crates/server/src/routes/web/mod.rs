use axum::{
  Router,
  extract::{Request, State},
  http::Uri,
  response::IntoResponse,
  routing::any,
};
use r2s_config::server::FrontendServeType;
use tower_http::services::{ServeDir, ServeFile};
use tracing::debug;

use crate::traits::{GlobalState, HTTPClient, ResponseError};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  let frontend_config = state.config.server.clone().unwrap_or_default().frontend;
  if let Some(frontend_config) = frontend_config {
    match frontend_config.serve_type {
      FrontendServeType::Static => Router::new().fallback_service(
        ServeDir::new(&frontend_config.path)
          .precompressed_gzip()
          .fallback(ServeFile::new(format!(
            "{}/index.html",
            frontend_config.path
          ))),
      ),
      FrontendServeType::Proxy => Router::new()
        .route("/", any(proxy_to_frontend_server))
        .route("/{*path}", any(proxy_to_frontend_server)),
    }
  } else {
    Router::new()
      .route("/", any(no_frontend_proxy))
      .route("/{*path}", any(no_frontend_proxy))
  }
}

async fn proxy_to_frontend_server(
  State(state): State<GlobalState>, State(client): State<HTTPClient>, mut req: Request,
) -> Result<impl IntoResponse, ResponseError> {
  // debug!("Proxying frontend request: {:?}", req);
  let frontend_config = state.config.server.clone().unwrap_or_default().frontend;
  let frontend_path = match frontend_config {
    Some(frontend_config) => frontend_config.path,
    None => {
      return Err(ResponseError::PreconditionFailed(String::from(
        "frontend proxy not set for Rhythm Arena; please contact the website operator",
      )));
    }
  };
  let path = req.uri().path();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|pq| pq.as_str())
    .unwrap_or(path);
  let uri = format!("{frontend_path}{path_query}");
  *req.uri_mut() = Uri::try_from(uri).unwrap();
  req.headers_mut().remove("host");

  let resp = client
    .request(req)
    .await
    .map_err(|err| ResponseError::BadRequest(format!("frontend proxy failed: {err}")))?
    .into_response();
  debug!(?resp, "proxying frontend request");
  Ok(resp)
}

async fn no_frontend_proxy() -> Result<(), ResponseError> {
  Err(ResponseError::PreconditionFailed(String::from(
    "frontend proxy not set for Rhythm Arena; please contact the website operator",
  )))
}
