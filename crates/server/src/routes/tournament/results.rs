use axum::{
  Extension, Json,
  extract::{Path, State},
  response::IntoResponse,
};
use chrono::{DateTime, Utc};
use r2s_database::{
  chart, registration,
  result::{self, ResultStatus},
  result_review, team_member,
  tournament::{EvidencePolicy, Lifecycle},
};
use r2s_migrator::Database;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
  QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{access, scoring};
use crate::{middleware::auth::Token, traits::ResponseError};

#[derive(Clone, Deserialize)]
pub struct ResultInput {
  pub registration_id: Option<i64>,
  pub chart_id: i64,
  pub score: i64,
  pub accuracy_millionths: i64,
  #[serde(default)]
  pub max_combo: i32,
  #[serde(default)]
  pub full_combo: bool,
  #[serde(default)]
  pub all_perfect: bool,
  #[serde(default = "empty_object")]
  pub judgments: Value,
  #[serde(default = "empty_object")]
  pub metrics: Value,
  pub played_at: i64,
  pub evidence: Option<String>,
}

fn empty_object() -> Value {
  json!({})
}

pub async fn list(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  access::tournament(&db, tournament_id).await?;
  let mut query = result::Entity::find().filter(result::Column::TournamentId.eq(tournament_id));
  if access::role(&db, tournament_id, &token).await?.is_none() {
    let registration = registration::Entity::find()
      .filter(registration::Column::TournamentId.eq(tournament_id))
      .filter(registration::Column::UserId.eq(token.id))
      .one(&db.conn)
      .await?
      .ok_or_else(|| ResponseError::Forbidden("registration required".to_owned()))?;
    query = query.filter(result::Column::RegistrationId.eq(registration.id));
  }
  Ok(Json(
    query
      .order_by_desc(result::Column::CreatedAt)
      .all(&db.conn)
      .await?,
  ))
}

async fn prepare(
  db: &Database, tournament_id: i64, token: &Token, input: ResultInput, staff_entry: bool,
) -> Result<result::ActiveModel, ResponseError> {
  if input.score < 0
    || !(0..=100_000_000).contains(&input.accuracy_millionths)
    || input.max_combo < 0
  {
    return Err(ResponseError::BadRequest(
      "score, accuracy, or combo is out of range".to_owned(),
    ));
  }
  let tournament = access::tournament(db, tournament_id).await?;
  if (!staff_entry && tournament.lifecycle != Lifecycle::Running)
    || (staff_entry && !matches!(tournament.lifecycle, Lifecycle::Running | Lifecycle::Review))
  {
    return Err(ResponseError::Conflict(
      "results can only be submitted while the tournament is running or under review".to_owned(),
    ));
  }
  let chart = chart::Entity::find_by_id(input.chart_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::BadRequest("chart does not belong to tournament".to_owned()))?;
  let registration = if staff_entry {
    if let Some(id) = input.registration_id {
      registration::Entity::find_by_id(id)
        .one(&db.conn)
        .await?
        .filter(|row| row.tournament_id == tournament_id)
    } else {
      registration::Entity::find()
        .filter(registration::Column::TournamentId.eq(tournament_id))
        .filter(registration::Column::UserId.eq(token.id))
        .one(&db.conn)
        .await?
    }
  } else {
    registration::Entity::find()
      .filter(registration::Column::TournamentId.eq(tournament_id))
      .filter(registration::Column::UserId.eq(token.id))
      .one(&db.conn)
      .await?
  }
  .ok_or_else(|| ResponseError::Forbidden("approved registration required".to_owned()))?;
  if registration.status != registration::RegistrationStatus::Approved {
    return Err(ResponseError::Forbidden(
      "approved registration required".to_owned(),
    ));
  }
  match (staff_entry, tournament.evidence_policy) {
    (false, EvidencePolicy::Required) if input.evidence.as_deref().unwrap_or("").is_empty() => {
      return Err(ResponseError::BadRequest(
        "screenshot evidence is required".to_owned(),
      ));
    }
    (false, EvidencePolicy::Disabled) if input.evidence.is_some() => {
      return Err(ResponseError::BadRequest(
        "screenshot evidence is disabled".to_owned(),
      ));
    }
    _ => {}
  }
  let team_id = team_member::Entity::find()
    .filter(team_member::Column::TournamentId.eq(tournament_id))
    .filter(team_member::Column::RegistrationId.eq(registration.id))
    .one(&db.conn)
    .await?
    .map(|row| row.team_id);
  let _ = chart;
  Ok(result::ActiveModel {
    id: Default::default(),
    tournament_id: Set(tournament_id),
    chart_id: Set(input.chart_id),
    registration_id: Set(registration.id),
    team_id_snapshot: Set(team_id),
    submitted_by: Set(token.id),
    score: Set(input.score),
    accuracy_millionths: Set(input.accuracy_millionths),
    max_combo: Set(input.max_combo),
    full_combo: Set(input.full_combo),
    all_perfect: Set(input.all_perfect),
    judgments: Set(input.judgments),
    metrics: Set(input.metrics),
    played_at: Set(
      DateTime::from_timestamp(input.played_at, 0)
        .ok_or_else(|| ResponseError::BadRequest("invalid played_at timestamp".to_owned()))?,
    ),
    evidence: Set(input.evidence),
    status: Set(if staff_entry {
      ResultStatus::Approved
    } else {
      ResultStatus::Pending
    }),
    replaces_result_id: Set(None),
    created_at: Set(Utc::now()),
  })
}

pub async fn submit(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<ResultInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  let staff_entry = access::role(&db, tournament_id, &token).await?.is_some();
  let row = prepare(&db, tournament_id, &token, input, staff_entry)
    .await?
    .insert(&db.conn)
    .await?;
  if row.status == ResultStatus::Approved {
    scoring::recompute_now(&db, tournament_id).await?;
  }
  Ok(Json(row))
}

#[derive(Serialize)]
pub struct PreviewRow {
  row: usize,
  valid: bool,
  error: Option<String>,
}

pub async fn preview_import(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(rows): Json<Vec<ResultInput>>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_judge(&db, tournament_id, &token).await?;
  let mut preview = Vec::with_capacity(rows.len());
  for (index, row) in rows.into_iter().enumerate() {
    match prepare(&db, tournament_id, &token, row, true).await {
      Ok(_) => preview.push(PreviewRow {
        row: index + 1,
        valid: true,
        error: None,
      }),
      Err(error) => preview.push(PreviewRow {
        row: index + 1,
        valid: false,
        error: Some(error.to_string()),
      }),
    }
  }
  Ok(Json(preview))
}

pub async fn commit_import(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(rows): Json<Vec<ResultInput>>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_judge(&db, tournament_id, &token).await?;
  if rows.is_empty() {
    return Err(ResponseError::BadRequest("import is empty".to_owned()));
  }
  let mut prepared = Vec::with_capacity(rows.len());
  for row in rows {
    prepared.push(prepare(&db, tournament_id, &token, row, true).await?);
  }
  let txn = db.conn.begin().await?;
  let mut inserted = Vec::with_capacity(prepared.len());
  for row in prepared {
    inserted.push(row.insert(&txn).await?);
  }
  txn.commit().await?;
  scoring::recompute_now(&db, tournament_id).await?;
  Ok(Json(inserted))
}

#[derive(Deserialize)]
pub struct ReviewInput {
  status: ResultStatus,
  reason: Option<String>,
}

fn valid_transition(from: ResultStatus, to: ResultStatus) -> bool {
  matches!(
    (from, to),
    (
      ResultStatus::Pending,
      ResultStatus::Approved | ResultStatus::Rejected | ResultStatus::Voided
    ) | (
      ResultStatus::Approved | ResultStatus::Rejected,
      ResultStatus::Voided
    )
  )
}

pub async fn review(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, result_id)): Path<(i64, i64)>, Json(input): Json<ReviewInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_judge(&db, tournament_id, &token).await?;
  let current = result::Entity::find_by_id(result_id)
    .one(&db.conn)
    .await?
    .filter(|r| r.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("result not found".to_owned()))?;
  if !valid_transition(current.status, input.status) {
    return Err(ResponseError::Conflict(
      "invalid result status transition".to_owned(),
    ));
  }
  let from = current.status;
  let txn = db.conn.begin().await?;
  let mut active = current.into_active_model();
  active.status = Set(input.status);
  let updated = active.update(&txn).await?;
  result_review::ActiveModel {
    id: Default::default(),
    result_id: Set(result_id),
    reviewer_id: Set(token.id),
    from_status: Set(from),
    to_status: Set(input.status),
    reason: Set(input.reason),
    created_at: Set(Utc::now()),
  }
  .insert(&txn)
  .await?;
  txn.commit().await?;
  if from == ResultStatus::Approved || input.status == ResultStatus::Approved {
    scoring::recompute_now(&db, tournament_id).await?;
  }
  Ok(Json(updated))
}

pub async fn correct(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, result_id)): Path<(i64, i64)>, Json(input): Json<ResultInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_judge(&db, tournament_id, &token).await?;
  let old = result::Entity::find_by_id(result_id)
    .one(&db.conn)
    .await?
    .filter(|r| r.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("result not found".to_owned()))?;
  if old.status == ResultStatus::Voided {
    return Err(ResponseError::Conflict(
      "voided result cannot be corrected again".to_owned(),
    ));
  }
  let from = old.status;
  let mut replacement = prepare(&db, tournament_id, &token, input, true).await?;
  replacement.replaces_result_id = Set(Some(old.id));
  let txn = db.conn.begin().await?;
  let mut old_active = old.into_active_model();
  old_active.status = Set(ResultStatus::Voided);
  old_active.update(&txn).await?;
  result_review::ActiveModel {
    id: Default::default(),
    result_id: Set(result_id),
    reviewer_id: Set(token.id),
    from_status: Set(from),
    to_status: Set(ResultStatus::Voided),
    reason: Set(Some("corrected by replacement".to_owned())),
    created_at: Set(Utc::now()),
  }
  .insert(&txn)
  .await?;
  let new_row = replacement.insert(&txn).await?;
  txn.commit().await?;
  scoring::recompute_now(&db, tournament_id).await?;
  Ok(Json(new_row))
}

#[cfg(test)]
mod tests {
  use r2s_database::result::ResultStatus;

  use super::valid_transition;

  #[test]
  fn result_state_machine_is_append_only_after_review() {
    assert!(valid_transition(
      ResultStatus::Pending,
      ResultStatus::Approved
    ));
    assert!(valid_transition(
      ResultStatus::Approved,
      ResultStatus::Voided
    ));
    assert!(!valid_transition(
      ResultStatus::Rejected,
      ResultStatus::Approved
    ));
    assert!(!valid_transition(
      ResultStatus::Voided,
      ResultStatus::Pending
    ));
  }
}
