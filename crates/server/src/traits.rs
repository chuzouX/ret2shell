use axum::{
  body::Body,
  extract::FromRef,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use hyper_util::client::legacy::{Client as HyperLegacyClient, connect::HttpConnector};
use r2s_auditor::Auditor;
use r2s_cache::Cache;
use r2s_config::GlobalConfig;
use r2s_database::DbErr;
use r2s_engine::Engine;
use r2s_event::EventManager;
use r2s_media::Media;
use r2s_migrator::Database;
use r2s_oauth::OAuth;
use r2s_queue::Queue;
use thiserror::Error;
use tracing::{error, warn};

pub type HTTPClient = HyperLegacyClient<HttpConnector, Body>;

#[derive(Clone, FromRef)]
pub struct GlobalState {
  pub config: GlobalConfig,
  pub requestor: HTTPClient,
  pub db: Database,
  pub cache: Cache,
  pub auditor: Auditor,
  pub engine: Engine,
  pub queue: Queue,
  pub oauth: OAuth,
  pub media: Media,
  pub event: EventManager,
  pub version: String,
}

#[derive(Debug, Error)]
pub enum ResponseError {
  #[error("internal server error: {0}")]
  InternalServerError(String),
  #[error("unauthorized: {0}")]
  Unauthorized(String),
  #[error("bad request: {0}")]
  BadRequest(String),
  #[error("forbidden: {0}")]
  Forbidden(String),
  #[error("not found: {0}")]
  NotFound(String),
  #[error("resource is outdated: {0}")]
  Gone(String),
  #[error("conflict: {0}")]
  Conflict(String),
  #[error("precondition failed: {0}")]
  PreconditionFailed(String),
  #[error("too many requests: {0}")]
  TooManyRequests(String),
  #[error("database error: {0}")]
  DatabaseError(#[from] r2s_database::DbErr),
  #[error("cache error: {0}")]
  CacheError(#[from] r2s_cache::CacheError),
  #[error("queue error: {0}")]
  QueueError(#[from] r2s_queue::QueueError),
  #[error("captcha error: {0}")]
  CaptchaError(#[from] r2s_captcha::CaptchaError),
  #[error("password hashing error: {0}")]
  PasswordHashError(#[from] crate::utility::password::PasswordHashingError),
  #[error("serialize error: {0}")]
  SerializeError(#[from] serde_json::Error),
  #[error("media storage error: {0}")]
  MediaError(#[from] r2s_media::MediaError),
  #[error("file io error: {0}")]
  FileIoError(#[from] std::io::Error),
  #[error("oauth error: {0}")]
  OAuthError(#[from] r2s_oauth::OAuthError),
  #[error("phira error: {0}")]
  PhiraError(#[from] r2s_oauth::phira::PhiraError),
  #[error("script engine error: {0}")]
  EngineError(#[from] r2s_engine::EngineError),
  #[error("string decode error: {0}")]
  StringDecodeError(#[from] std::string::FromUtf8Error),
}

macro_rules! log_with_resp {
  ($code:expr, $summary:expr, $detail:expr) => {{
    if ($code).is_server_error() {
      error!("{}: {}", $summary, $detail);
    } else {
      warn!("{}: {}", $summary, $detail);
    }
    ($code, $summary)
  }};
}

impl IntoResponse for ResponseError {
  fn into_response(self) -> Response<Body> {
    let (status, message) = match self {
      ResponseError::InternalServerError(summary) => (StatusCode::INTERNAL_SERVER_ERROR, summary),
      ResponseError::Unauthorized(summary) => (StatusCode::UNAUTHORIZED, summary),
      ResponseError::BadRequest(summary) => (StatusCode::BAD_REQUEST, summary),
      ResponseError::Forbidden(summary) => (StatusCode::FORBIDDEN, summary),

      ResponseError::NotFound(summary) => (StatusCode::NOT_FOUND, summary),
      ResponseError::Conflict(summary) => (StatusCode::CONFLICT, summary),
      ResponseError::TooManyRequests(summary) => (StatusCode::TOO_MANY_REQUESTS, summary),
      ResponseError::PreconditionFailed(summary) => (StatusCode::PRECONDITION_FAILED, summary),
      ResponseError::DatabaseError(e) => match e {
        DbErr::RecordNotFound(s) => (StatusCode::NOT_FOUND, format!("record not found: {s}")),
        DbErr::Json(_) => (
          StatusCode::INTERNAL_SERVER_ERROR,
          "data cruptted".to_owned(),
        ),
        _ => log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "database internal error".to_owned(),
          e.to_string()
        ),
      },
      ResponseError::Gone(summary) => (StatusCode::GONE, summary),
      ResponseError::CacheError(e) => match e {
        r2s_cache::CacheError::DomainNeeded(s) => {
          log_with_resp!(StatusCode::BAD_REQUEST, "cache domain needed".to_owned(), s)
        }
        r2s_cache::CacheError::ConfigNeeded => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "missing cache".to_owned(),
            "cache config is not set yet"
          )
        }
        r2s_cache::CacheError::Redis(_) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cache server seems down".to_owned(),
            "cache server seems down"
          )
        }
        r2s_cache::CacheError::Serde(_) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cached data consistency is compromised".to_owned(),
            "failed to serialize data"
          )
        }
        _ => log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "cache internal error".to_owned(),
          e.to_string()
        ),
      },
      ResponseError::QueueError(e) => match e {
        r2s_queue::QueueError::PublishError(s) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "message queue refused publishing".to_owned(),
            s
          )
        }
        _ => log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "queue internal error".to_owned(),
          e.to_string()
        ),
      },
      ResponseError::CaptchaError(e) => {
        log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to generate captcha".to_owned(),
          e.to_string()
        )
      }
      ResponseError::PasswordHashError(e) => {
        log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to hash password".to_owned(),
          e.to_string()
        )
      }
      ResponseError::SerializeError(e) => {
        log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to serialize data".to_owned(),
          e.to_string()
        )
      }
      ResponseError::MediaError(e) => match e {
        r2s_media::MediaError::ParseContentTypeError(e) => {
          log_with_resp!(
            StatusCode::BAD_REQUEST,
            "failed to parse content type".to_owned(),
            e.to_string()
          )
        }
        r2s_media::MediaError::UnsupportedFileType(s) => {
          log_with_resp!(
            StatusCode::BAD_REQUEST,
            "unsupported file type".to_owned(),
            s
          )
        }
        r2s_media::MediaError::MediaStoragePathNotConfigured => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "media storage path not configured".to_owned(),
            "media storage path is not set yet"
          )
        }
        _ => log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "media internal error".to_owned(),
          format!("media internal error: {e:?}")
        ),
      },
      ResponseError::FileIoError(e) => {
        log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "file io error".to_owned(),
          format!("failed to read/write file: {e:?}")
        )
      }
      ResponseError::OAuthError(e) => match e {
        r2s_oauth::OAuthError::NetworkError(_) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "missing OAuth config".to_owned(),
            "OAuth config is not set yet"
          )
        }
        _ => log_with_resp!(
          StatusCode::FORBIDDEN,
          "failed to login with 3rd account".to_owned(),
          format!("failed to login with 3rd account: {e:?}")
        ),
      },
      ResponseError::PhiraError(e) => match e {
        r2s_oauth::phira::PhiraError::Authentication => (
          StatusCode::FORBIDDEN,
          "phira email or password is wrong".to_owned(),
        ),
        r2s_oauth::phira::PhiraError::Request(_) => log_with_resp!(
          StatusCode::BAD_GATEWAY,
          "phira service is unavailable".to_owned(),
          "failed to request Phira"
        ),
        r2s_oauth::phira::PhiraError::InvalidResponse => log_with_resp!(
          StatusCode::BAD_GATEWAY,
          "invalid response from Phira".to_owned(),
          "Phira returned an invalid response"
        ),
      },
      ResponseError::EngineError(e) => match e {
        r2s_engine::EngineError::MissingScript(_) => {
          log_with_resp!(
            StatusCode::PRECONDITION_FAILED,
            "missing scoring script".to_owned(),
            format!("missing scoring script: {e:?}")
          )
        }
        r2s_engine::EngineError::MissingFunction(e) => {
          log_with_resp!(
            StatusCode::PRECONDITION_FAILED,
            format!("missing script function: {e:?}"),
            format!("missing script function: {e:?}")
          )
        }
        r2s_engine::EngineError::CompileError(e) => {
          log_with_resp!(
            StatusCode::PRECONDITION_FAILED,
            "failed to compile scoring script".to_owned(),
            format!("failed to compile scoring script: {e:?}")
          )
        }
        r2s_engine::EngineError::AllocError(e) => log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to build scoring script engine".to_owned(),
          format!("failed to build scoring script engine: {e:?}")
        ),
        r2s_engine::EngineError::ExecError(e) => log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to execute scoring script".to_owned(),
          format!("failed to execute scoring script: {e:?}")
        ),
        r2s_engine::EngineError::MissingResultField(e) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "missing values in scoring script results".to_owned(),
            format!("missing values in scoring script results: {e:?}")
          )
        }
        r2s_engine::EngineError::BuildError(e) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to build scoring script unit".to_owned(),
            format!("failed to build scoring script unit: {e:?}")
          )
        }
        r2s_engine::EngineError::SourceError(e) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load scoring script source".to_owned(),
            format!("failed to load scoring script source: {e:?}")
          )
        }
        r2s_engine::EngineError::RuneError(e) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "error occurred in scoring script context; check server logs".to_owned(),
            format!("error occurred in scoring script context: {e:?}")
          )
        }
        r2s_engine::EngineError::RuneRuntimeError(e) => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "error occurred in scoring script engine; check server logs".to_owned(),
            format!("error occurred in scoring script engine: {e:?}")
          )
        }
        r2s_engine::EngineError::ScriptError(_) => (
          StatusCode::PRECONDITION_FAILED,
          "scoring script rejected the input".to_owned(),
        ),
        _ => {
          log_with_resp!(
            StatusCode::INTERNAL_SERVER_ERROR,
            "scoring script internal error".to_owned(),
            e.to_string()
          )
        }
      },
      ResponseError::StringDecodeError(e) => {
        log_with_resp!(
          StatusCode::INTERNAL_SERVER_ERROR,
          "failed to decode string".to_owned(),
          e.to_string()
        )
      }
    };

    Response::builder()
      .status(status)
      .header("Content-Type", "text/plain")
      .body(message.into())
      .unwrap()
  }
}
