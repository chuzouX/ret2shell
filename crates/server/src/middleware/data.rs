use axum::{
  Extension,
  extract::{Request, State},
  middleware::Next,
  response::IntoResponse,
};
use r2s_cache::Cache;
use r2s_config::GlobalConfig;
use r2s_migrator::Database;
use tracing::debug;

use super::auth::Token;
use crate::traits::ResponseError;

pub async fn prepare_config(
  State(ref db): State<Database>, State(ref cache): State<Cache>,
  State(config): State<GlobalConfig>, mut req: Request, next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  match cache.at("platform").get("config").await? {
    Some(info) => {
      req
        .extensions_mut()
        .insert::<r2s_database::config::Model>(info);
    }
    None => {
      let dynamic_config = r2s_database::config::get(&db.conn).await?;
      debug!(
        ?dynamic_config,
        static_config=?config,
        "merging static and dynamic config"
      );
      let dynamic_config = dynamic_config.unwrap_or_default().merge(config);
      cache.at("platform").set("config", &dynamic_config).await?;
      req
        .extensions_mut()
        .insert::<r2s_database::config::Model>(dynamic_config);
    }
  }
  Ok(next.run(req).await)
}

pub async fn prepare_user_info(
  State(ref db): State<Database>, Extension(token): Extension<Token>, mut req: Request, next: Next,
) -> Result<impl IntoResponse, ResponseError> {
  let user = r2s_database::user::get(&db.conn, token.id).await?;
  match user {
    Some(user) => {
      req
        .extensions_mut()
        .insert::<r2s_database::user::Model>(user);
      Ok(next.run(req).await)
    }
    None => Err(ResponseError::Unauthorized("please login first".into())),
  }
}

macro_rules! get_path_param_i64 {
  ($key:expr, $params:expr) => {{
    let key = $params
      .get($key)
      .ok_or(crate::traits::ResponseError::PreconditionFailed(format!(
        "missing {}",
        $key
      )))?;
    let key = key
      .parse::<i64>()
      .map_err(|_| crate::traits::ResponseError::PreconditionFailed(format!("invalid {}", $key)))?;
    key
  }};
}

pub(crate) use get_path_param_i64;

/// Prepare data for the request
///
/// * `$model` - the model name
/// * `$cached` - whether to cache the result
///
/// Remember to refresh cache when update the data!
macro_rules! prepare_data {
  ($model:ident, $cached: expr, $($trace:tt),*) => {
    |
      axum::extract::State(db): axum::extract::State<r2s_migrator::Database>,
      axum::extract::State(cache): axum::extract::State<r2s_cache::Cache>,
      axum::extract::Path(params): axum::extract::Path<std::collections::HashMap<String, String>>,
      mut req: axum::extract::Request,
      next: axum::middleware::Next
    | async move {
      let id = crate::middleware::data::get_path_param_i64!(stringify!($model), &params);
      let data = if $cached {
        match cache.at(stringify!($model)).get(id).await? {
          Some(info) => Some(info),
          None => {
            let data = r2s_database::$model::get(&db.conn, id).await?;
            match data {
              Some(data) => {
                cache
                  .at(stringify!($model))
                  .set_ex(id.to_string(), &data, 60 * 60 * 24)
                  .await?;
                Some(data)
              }
              None => None,
            }
          }
        }
      } else {
        r2s_database::$model::get(&db.conn, id).await?
      };
      match data {
        Some(data) => {
          // if trace is enabled, add trace
          let traced: Vec<&str> = vec![$(stringify!($trace)),*];
          let traced = traced.len() > 0;
            req
              .extensions_mut()
              .insert::<r2s_database::$model::Model>(data.clone());
          if traced {
             $(
               tracing::Span::current().record(concat!("data-", stringify!($model), "-", stringify!($trace)), &data.$trace);
             )*
          }
          Ok(next.run(req).await)
        }
        None => Err(crate::traits::ResponseError::NotFound(format!(
          "{} not found",
          stringify!($model)
        ))),
      }
    }
  };
}

pub(crate) use prepare_data;
