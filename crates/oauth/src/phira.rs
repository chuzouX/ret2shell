use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

const PHIRA_URL: &str = "https://phira.5wyxi.com";

pub fn debug_event(hypothesis_id: &str, message: &str, data: Value) {
  let payload = serde_json::json!({
    "sessionId": "phira-import-db-error",
    "runId": "pre-fix",
    "hypothesisId": hypothesis_id,
    "location": "crates/oauth/src/phira.rs",
    "msg": format!("[DEBUG] {message}"),
    "data": data,
  });
  tokio::spawn(async move {
    let _ = reqwest::Client::new()
      .post("http://127.0.0.1:7777/event")
      .json(&payload)
      .send()
      .await;
  });
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chart {
  pub id: i64,
  pub name: String,
  pub level: String,
  pub difficulty: f64,
  pub charter: String,
  pub composer: String,
  pub illustration: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(flatten)]
  pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct ChartList {
  count: i64,
  #[serde(alias = "result")]
  results: Vec<Chart>,
}

fn client() -> Result<reqwest::Client, PhiraError> {
  reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()
    .map_err(PhiraError::Request)
}

pub async fn get_chart(id: i64) -> Result<Chart, PhiraError> {
  // #region debug-point D:request
  debug_event(
    "D",
    "requesting phira chart",
    serde_json::json!({ "id": id }),
  );
  // #endregion
  let response = client()?
    .get(format!("{PHIRA_URL}/chart/{id}"))
    .header(reqwest::header::ACCEPT, "application/json")
    .send()
    .await
    .map_err(PhiraError::Request)?;
  if !response.status().is_success() {
    // #region debug-point A:status
    debug_event(
      "A",
      "phira chart request returned non-success",
      serde_json::json!({ "status": response.status().as_u16() }),
    );
    // #endregion
    return Err(PhiraError::InvalidResponse);
  }
  let chart = response
    .json::<Chart>()
    .await
    .map_err(|_| PhiraError::InvalidResponse)?;
  // #region debug-point A:parsed
  debug_event(
    "A",
    "phira chart response parsed",
    serde_json::json!({ "id": chart.id, "has_name": !chart.name.trim().is_empty() }),
  );
  // #endregion
  Ok(chart)
}

pub async fn get_popular(page: u32, page_num: u32) -> Result<(i64, Vec<Chart>), PhiraError> {
  let response = client()?
    .get(format!(
      "{PHIRA_URL}/chart?page={}&pageNum={}&type=-1",
      page.max(1),
      page_num.clamp(1, 30)
    ))
    .header(reqwest::header::ACCEPT, "application/json")
    .send()
    .await
    .map_err(PhiraError::Request)?;
  if !response.status().is_success() {
    return Err(PhiraError::InvalidResponse);
  }
  let list = response
    .json::<ChartList>()
    .await
    .map_err(|_| PhiraError::InvalidResponse)?;
  Ok((list.count, list.results))
}

pub async fn authenticate(email: &str, password: &str) -> Result<Identity, PhiraError> {
  let client = client()?;

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
