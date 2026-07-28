use serde::{Deserialize, Serialize};
use thiserror::Error;

const PHIRA_URL: &str = "https://phira.5wyxi.com";

#[derive(Debug, Error)]
pub enum PhiraError {
  #[error("phira authentication failed")]
  Authentication,
  #[error("phira service request failed")]
  Request(#[source] reqwest::Error),
  #[error("phira service returned an invalid response")]
  InvalidResponse,
}

#[derive(Clone, Debug, Serialize)]
struct LoginRequest<'a> {
  email: &'a str,
  password: &'a str,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
  id: i64,
  token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Identity {
  pub id: i64,
  pub name: String,
  pub avatar: Option<String>,
}

pub async fn authenticate(email: &str, password: &str) -> Result<Identity, PhiraError> {
  let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()
    .map_err(PhiraError::Request)?;

  let response = client
    .post(format!("{PHIRA_URL}/login"))
    .header(reqwest::header::ACCEPT, "application/json")
    .json(&LoginRequest { email, password })
    .send()
    .await
    .map_err(PhiraError::Request)?;

  if response.status() == reqwest::StatusCode::UNAUTHORIZED
    || response.status() == reqwest::StatusCode::FORBIDDEN
  {
    return Err(PhiraError::Authentication);
  }
  if !response.status().is_success() {
    return Err(PhiraError::InvalidResponse);
  }

  let body = response
    .json::<serde_json::Value>()
    .await
    .map_err(|_| PhiraError::InvalidResponse)?;
  if body
    .get("error")
    .and_then(serde_json::Value::as_str)
    .is_some()
  {
    return Err(PhiraError::Authentication);
  }
  let login =
    serde_json::from_value::<LoginResponse>(body).map_err(|_| PhiraError::InvalidResponse)?;
  if login.token.is_empty() {
    return Err(PhiraError::InvalidResponse);
  }

  let response = client
    .get(format!("{PHIRA_URL}/me"))
    .header(reqwest::header::ACCEPT, "application/json")
    .bearer_auth(&login.token)
    .send()
    .await
    .map_err(PhiraError::Request)?;

  if response.status() == reqwest::StatusCode::UNAUTHORIZED
    || response.status() == reqwest::StatusCode::FORBIDDEN
  {
    return Err(PhiraError::Authentication);
  }
  if !response.status().is_success() {
    return Err(PhiraError::InvalidResponse);
  }

  let identity = response
    .json::<Identity>()
    .await
    .map_err(|_| PhiraError::InvalidResponse)?;
  if identity.id != login.id || identity.name.trim().is_empty() {
    return Err(PhiraError::InvalidResponse);
  }

  Ok(identity)
}
