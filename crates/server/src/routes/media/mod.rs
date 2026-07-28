use axum::{
  Extension, Json, Router,
  body::Body,
  extract::{DefaultBodyLimit, Multipart, Query, State},
  http::{HeaderMap, HeaderValue, StatusCode},
  middleware,
  response::IntoResponse,
  routing::{get, post},
};
use axum_extra::{TypedHeader, headers::Host};
use futures::TryStreamExt;
use r2s_cache::Cache;
use r2s_database::{config, media, user::Permission};
use r2s_media::Media;
use r2s_migrator::Database;
use serde::Deserialize;
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::warn;

use crate::{
  middleware::auth::{self, Token},
  traits::{GlobalState, ResponseError},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/", post(upload_media))
    // media image size limited to 10MB
    .route_layer(DefaultBodyLimit::max(1024 * 1024 * 10))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Basic,
      Permission::Verified
    )))
    .route("/", get(get_media))
}

#[derive(Deserialize)]
struct MediaQuery {
  hash: String,
}

async fn get_media(
  TypedHeader(host): TypedHeader<Host>, State(ref db): State<Database>,
  State(ref media): State<Media>, Extension(config): Extension<config::Model>,
  Query(query): Query<MediaQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  if config.media.is_some_and(|media| media.anti_theft)
    && config
      .server
      .is_some_and(|server| server.external_domain != host.hostname())
    && host.hostname() != "localhost"
  {
    warn!(
      origin=?host.hostname(),
      "origin is not allowed to access media",
    );
    return Err(ResponseError::Forbidden("origin is not allowed".to_owned()));
  }
  let model = media::get_by_hash(&db.conn, &query.hash).await?;
  if model.is_none() {
    return Err(ResponseError::NotFound("media".to_string()));
  }
  let file = media.get(&query.hash).await?;
  let stream = ReaderStream::new(file);
  let mut headers = HeaderMap::new();
  headers.insert(
    "Content-Type",
    media
      .get_mime_type(&query.hash)?
      .parse::<HeaderValue>()
      .map_err(|e| {
        warn!(error=?e, "failed to parse mime type");
        ResponseError::InternalServerError("failed to parse mime type".to_string())
      })?,
  );
  headers.insert(
    "Content-Security-Policy",
    HeaderValue::from_static(
      "script-src 'none'; object-src 'none'; report-uri /cspreport; sandbox",
    ),
  );
  Ok((StatusCode::OK, headers, Body::from_stream(stream)))
}

#[derive(Deserialize)]
struct UploadMediaQuery {
  thumbnail: Option<bool>,
}

async fn upload_media(
  State(ref media): State<Media>, State(ref cache): State<Cache>, State(ref db): State<Database>,
  Extension(token): Extension<Token>, Extension(config): Extension<config::Model>,
  Query(query): Query<UploadMediaQuery>, mut multipart: Multipart,
) -> Result<impl IntoResponse, ResponseError> {
  let uploads: Option<i32> = cache.at("media").get(token.id).await?;
  if !token.permissions.0.contains(&Permission::Bulletin)
    && !token.permissions.0.contains(&Permission::Tournament)
    && !token.permissions.0.contains(&Permission::Wiki)
    && uploads.is_some_and(|u| config.media.is_some_and(|m| u >= m.limit))
  {
    warn!("user has reached the upload limit");
    return Err(ResponseError::TooManyRequests(
      "too many uploads".to_owned(),
    ));
  } else if uploads.is_none()
    && !token.permissions.0.contains(&Permission::Bulletin)
    && !token.permissions.0.contains(&Permission::Calendar)
    && !token.permissions.0.contains(&Permission::DevOps)
    && !token.permissions.0.contains(&Permission::Tournament)
    && !token.permissions.0.contains(&Permission::Host)
    && !token.permissions.0.contains(&Permission::Wiki)
  {
    cache
      .at("media")
      .set_ex(token.id, 1, 60 * 60 * 24 * 7)
      .await?;
  } else {
    cache.at("media").incr(token.id).await?;
  }

  if let Some(field) = multipart
    .next_field()
    .await
    .map_err(|err| ResponseError::BadRequest(err.to_string()))?
  {
    let reader = StreamReader::new(field.map_err(std::io::Error::other));
    let model = media.save(reader).await?;
    if query.thumbnail.unwrap_or(false) {
      media.make_thumbnail(&model.hash, 128).await?;
    }
    if let Some(result) = media::get_by_hash(&db.conn, &model.hash).await? {
      Ok(Json(result))
    } else {
      let model = media::create(
        &db.conn,
        media::Model {
          id: 0,
          uploader_id: token.id,
          ..model
        },
      )
      .await?;
      Ok(Json(model))
    }
  } else {
    Err(ResponseError::BadRequest("no file uploaded".to_owned()))
  }
}
